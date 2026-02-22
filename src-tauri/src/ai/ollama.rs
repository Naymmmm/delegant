use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

use super::types::{AiResponse, ContentBlock, Message, ToolDefinition};

pub struct OllamaClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OllamaClient {
    pub fn new(api_key: &str, model: &str, base_url: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
            base_url: normalize_base_url(base_url),
        }
    }

    pub async fn send(
        &self,
        system: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> AppResult<AiResponse> {
        // Convert to OpenAI-compatible message format
        let mut api_messages: Vec<serde_json::Value> = vec![json!({
            "role": "system",
            "content": system
        })];

        for msg in messages {
            let mut parts: Vec<serde_json::Value> = Vec::new();

            if msg.role == "user" {
                let mut tool_results = Vec::new();
                let mut regular_parts = Vec::new();

                for block in &msg.content {
                    match block {
                        ContentBlock::ToolResult {
                            tool_use_id,
                            content,
                            ..
                        } => {
                            tool_results.push(json!({
                                "role": "tool",
                                "tool_call_id": tool_use_id,
                                "content": content
                            }));
                        }
                        ContentBlock::Text { text } => {
                            regular_parts.push(json!({
                                "type": "text",
                                "text": text
                            }));
                        }
                        ContentBlock::Image { source } => {
                            regular_parts.push(json!({
                                "type": "image_url",
                                "image_url": {
                                    "url": format!("data:{};base64,{}", source.media_type, source.data)
                                }
                            }));
                        }
                        _ => {}
                    }
                }

                for tr in tool_results {
                    api_messages.push(tr);
                }

                if !regular_parts.is_empty() {
                    api_messages.push(json!({
                        "role": "user",
                        "content": regular_parts
                    }));
                }
                continue;
            }

            if msg.role == "assistant" {
                let mut text_parts = Vec::new();
                let mut tool_calls = Vec::new();

                for block in &msg.content {
                    match block {
                        ContentBlock::Text { text } => {
                            text_parts.push(text.clone());
                        }
                        ContentBlock::ToolUse { id, name, input } => {
                            tool_calls.push(json!({
                                "id": id,
                                "type": "function",
                                "function": {
                                    "name": name,
                                    "arguments": serde_json::to_string(input).unwrap_or_default()
                                }
                            }));
                        }
                        _ => {}
                    }
                }

                let mut assistant_msg = json!({
                    "role": "assistant",
                    "content": text_parts.join("\n")
                });
                if !tool_calls.is_empty() {
                    assistant_msg["tool_calls"] = json!(tool_calls);
                }
                api_messages.push(assistant_msg);
                continue;
            }

            for block in &msg.content {
                match block {
                    ContentBlock::Text { text } => {
                        parts.push(json!({
                            "type": "text",
                            "text": text
                        }));
                    }
                    ContentBlock::Image { source } => {
                        parts.push(json!({
                            "type": "image_url",
                            "image_url": {
                                "url": format!("data:{};base64,{}", source.media_type, source.data)
                            }
                        }));
                    }
                    _ => {}
                }
            }
            if !parts.is_empty() {
                api_messages.push(json!({
                    "role": msg.role,
                    "content": parts
                }));
            }
        }

        let api_tools: Vec<serde_json::Value> = tools
            .iter()
            .filter(|t| t.input_schema.is_some())
            .map(|tool| {
                json!({
                    "type": "function",
                    "function": {
                        "name": tool.name,
                        "description": tool.description.clone().unwrap_or_default(),
                        "parameters": tool.input_schema.clone().unwrap_or(json!({}))
                    }
                })
            })
            .collect();

        let mut body = json!({
            "model": self.model,
            "messages": api_messages,
            "stream": false
        });
        if !api_tools.is_empty() {
            body["tools"] = json!(api_tools);
        }

        let mut req = self
            .client
            .post(format!("{}/v1/chat/completions", self.base_url))
            .header("content-type", "application/json");
        if !self.api_key.trim().is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.api_key));
        }

        let resp = req.json(&body).send().await?;
        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(AppError::AiProvider(format!(
                "Ollama API error {}: {}",
                status, text
            )));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| AppError::AiProvider(e.to_string()))?;

        let choice = &parsed["choices"][0];
        let message = &choice["message"];
        let finish_reason = choice["finish_reason"].as_str().unwrap_or("stop");

        let mut content = Vec::new();

        if let Some(text) = message["content"].as_str() {
            if !text.is_empty() {
                content.push(ContentBlock::Text {
                    text: text.to_string(),
                });
            }
        } else if let Some(arr) = message["content"].as_array() {
            let joined = arr
                .iter()
                .filter_map(|part| part["text"].as_str())
                .collect::<Vec<_>>()
                .join("\n");
            if !joined.is_empty() {
                content.push(ContentBlock::Text { text: joined });
            }
        }

        if let Some(tool_calls) = message["tool_calls"].as_array() {
            for tc in tool_calls {
                let id = tc["id"]
                    .as_str()
                    .map(str::to_string)
                    .filter(|s| !s.is_empty())
                    .unwrap_or_else(|| format!("tool_{}", Uuid::new_v4()));
                let name = tc["function"]["name"].as_str().unwrap_or("").to_string();
                let args = &tc["function"]["arguments"];

                let input = if let Some(args_str) = args.as_str() {
                    serde_json::from_str(args_str).unwrap_or(json!({}))
                } else if args.is_object() {
                    args.clone()
                } else {
                    json!({})
                };

                content.push(ContentBlock::ToolUse { id, name, input });
            }
        }

        let stop_reason = if finish_reason == "tool_calls"
            || message["tool_calls"]
                .as_array()
                .map(|arr| !arr.is_empty())
                .unwrap_or(false)
        {
            "tool_use".to_string()
        } else {
            "end_turn".to_string()
        };

        Ok(AiResponse {
            content,
            stop_reason,
        })
    }
}

fn normalize_base_url(input: &str) -> String {
    let trimmed = input.trim();
    let mut url = if trimmed.is_empty() {
        "http://127.0.0.1:11434".to_string()
    } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    };

    while url.ends_with('/') {
        url.pop();
    }

    if url.ends_with("/v1") {
        url.truncate(url.len() - 3);
    }

    while url.ends_with('/') {
        url.pop();
    }

    url
}

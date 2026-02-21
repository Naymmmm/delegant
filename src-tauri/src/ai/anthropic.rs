use reqwest::Client;
use serde_json::json;

use crate::error::{AppError, AppResult};

use super::types::{AiResponse, ContentBlock, Message, ToolDefinition};

pub struct AnthropicClient {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicClient {
    pub fn new(api_key: &str, model: &str) -> Self {
        Self {
            client: Client::new(),
            api_key: api_key.to_string(),
            model: model.to_string(),
        }
    }

    pub async fn send(
        &self,
        system: &str,
        messages: &[Message],
        tools: &[ToolDefinition],
    ) -> AppResult<AiResponse> {
        // Build the Anthropic-format messages
        let api_messages: Vec<serde_json::Value> = messages
            .iter()
            .map(|msg| {
                let content: Vec<serde_json::Value> = msg
                    .content
                    .iter()
                    .map(|block| match block {
                        ContentBlock::Text { text } => json!({
                            "type": "text",
                            "text": text
                        }),
                        ContentBlock::Image { source } => json!({
                            "type": "image",
                            "source": {
                                "type": source.source_type,
                                "media_type": source.media_type,
                                "data": source.data
                            }
                        }),
                        ContentBlock::ToolUse { id, name, input } => json!({
                            "type": "tool_use",
                            "id": id,
                            "name": name,
                            "input": input
                        }),
                        ContentBlock::ToolResult {
                            tool_use_id,
                            content,
                            is_error,
                        } => {
                            let mut v = json!({
                                "type": "tool_result",
                                "tool_use_id": tool_use_id,
                                "content": content
                            });
                            if let Some(true) = is_error {
                                v["is_error"] = json!(true);
                            }
                            v
                        }
                    })
                    .collect();
                json!({
                    "role": msg.role,
                    "content": content
                })
            })
            .collect();

        // Build tools in Anthropic format
        let api_tools: Vec<serde_json::Value> = tools
            .iter()
            .map(|tool| {
                let mut t = json!({ "name": tool.name });
                if let Some(ref tool_type) = tool.tool_type {
                    t["type"] = json!(tool_type);
                }
                if let Some(ref desc) = tool.description {
                    t["description"] = json!(desc);
                }
                if let Some(ref schema) = tool.input_schema {
                    t["input_schema"] = schema.clone();
                }
                if let Some(w) = tool.display_width_px {
                    t["display_width_px"] = json!(w);
                }
                if let Some(h) = tool.display_height_px {
                    t["display_height_px"] = json!(h);
                }
                if let Some(n) = tool.display_number {
                    t["display_number"] = json!(n);
                }
                t
            })
            .collect();

        let body = json!({
            "model": self.model,
            "max_tokens": 4096,
            "system": system,
            "tools": api_tools,
            "messages": api_messages
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("anthropic-beta", "computer-use-2025-01-24")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        let text = resp.text().await?;

        if !status.is_success() {
            return Err(AppError::AiProvider(format!(
                "Anthropic API error {}: {}",
                status, text
            )));
        }

        let parsed: serde_json::Value =
            serde_json::from_str(&text).map_err(|e| AppError::AiProvider(e.to_string()))?;

        let stop_reason = parsed["stop_reason"]
            .as_str()
            .unwrap_or("end_turn")
            .to_string();

        let content_blocks = parsed["content"]
            .as_array()
            .ok_or_else(|| AppError::AiProvider("No content in response".into()))?;

        let mut content = Vec::new();
        for block in content_blocks {
            match block["type"].as_str() {
                Some("text") => {
                    content.push(ContentBlock::Text {
                        text: block["text"].as_str().unwrap_or("").to_string(),
                    });
                }
                Some("tool_use") => {
                    content.push(ContentBlock::ToolUse {
                        id: block["id"].as_str().unwrap_or("").to_string(),
                        name: block["name"].as_str().unwrap_or("").to_string(),
                        input: block["input"].clone(),
                    });
                }
                _ => {}
            }
        }

        Ok(AiResponse {
            content,
            stop_reason,
        })
    }
}

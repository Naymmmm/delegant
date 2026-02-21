use std::sync::Arc;

use serde_json::json;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

use crate::agent::action::{execute_action, parse_computer_action, AgentAction};
use crate::agent::history::trim_history;
use crate::agent::tools::build_tool_definitions;
use crate::ai::anthropic::AnthropicClient;
use crate::ai::openai::OpenAiClient;
use crate::ai::openrouter::OpenRouterClient;
use crate::ai::types::{AiResponse, ContentBlock, ImageSource, Message};
use crate::error::AppResult;
use crate::state::{AgentState, Settings};

const SYSTEM_PROMPT: &str = r#"You are an AI agent running on Delegant that controls a computer to accomplish tasks. You can see the screen via screenshots and perform actions using the available tools.

IMPORTANT GUIDELINES:
- Always take a screenshot first to see the current state of the screen before acting.
- Screenshots show the full screen. Coordinates are pixel positions from top-left (0,0).
- Be precise with coordinates when clicking — aim for the center of buttons, links, and text fields.
- Look carefully at the screenshot to identify clickable UI elements, menus, icons, and text.
- After performing an action, take a screenshot to verify the result before proceeding.
- If something doesn't work, try an alternative approach.
- Use bash/shell commands when they are more efficient than GUI interactions.
- When you believe the task is complete, say so clearly and stop using tools.
- In your text responses, include a JSON snippet estimating remaining time: {"estimated_seconds": N} where N is your best estimate of seconds remaining to complete the task. Update this estimate as you progress."#;

fn show_cursor_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("cursor-overlay") {
        let _ = win.show();
    }
}

fn hide_cursor_overlay(app: &AppHandle) {
    if let Some(win) = app.get_webview_window("cursor-overlay") {
        let _ = app.emit_to("cursor-overlay", "cursor-hide", ());
        let _ = win.hide();
    }
}

pub async fn run_agent_loop(
    task: String,
    cancel_token: CancellationToken,
    settings: Arc<RwLock<Settings>>,
    agent_state: Arc<Mutex<AgentState>>,
    app: AppHandle,
) -> AppResult<()> {
    let s = settings.read().await.clone();

    let tools = build_tool_definitions(&s.provider, s.display_width, s.display_height);

    // Show cursor overlay
    show_cursor_overlay(&app);

    // Initial user message with the task
    let mut messages: Vec<Message> = vec![Message {
        role: "user".to_string(),
        content: vec![ContentBlock::Text {
            text: format!(
                "Task: {}\n\nThe screen resolution is {}x{} pixels. Coordinates are [x, y] from the top-left corner. Please start by taking a screenshot to see the current state of the screen.",
                task, s.display_width, s.display_height
            ),
        }],
    }];

    let mut iteration: u32 = 0;
    let mut last_nodes: Option<Vec<crate::screen::a11y::A11yNode>> = None;

    loop {
        if cancel_token.is_cancelled() {
            hide_cursor_overlay(&app);
            return Ok(());
        }

        // Update iteration
        {
            let mut agent = agent_state.lock().await;
            agent.iteration = iteration;
        }

        // Send to AI
        let _ = app.emit("agent-thinking", json!({"text": "Thinking..."}));

        let response: AiResponse = match s.provider.as_str() {
            "openai" => {
                let client = OpenAiClient::new(&s.openai_api_key, &s.model);
                client.send(SYSTEM_PROMPT, &messages, &tools).await?
            }
            "openrouter" => {
                let client = OpenRouterClient::new(&s.openrouter_api_key, &s.model);
                client.send(SYSTEM_PROMPT, &messages, &tools).await?
            }
            _ => {
                let client = AnthropicClient::new(&s.anthropic_api_key, &s.model);
                client.send(SYSTEM_PROMPT, &messages, &tools).await?
            }
        };

        // Process response
        let mut assistant_blocks: Vec<ContentBlock> = Vec::new();
        let mut tool_results: Vec<ContentBlock> = Vec::new();
        let mut has_tool_use = false;

        for block in &response.content {
            match block {
                ContentBlock::Text { text } => {
                    assistant_blocks.push(block.clone());
                    let _ = app.emit("agent-message", json!({"text": text}));

                    // Extract estimated time
                    if let Some(start) = text.find("{\"estimated_seconds\"") {
                        if let Some(end) = text[start..].find('}') {
                            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(
                                &text[start..start + end + 1],
                            ) {
                                if let Some(secs) = parsed["estimated_seconds"].as_u64() {
                                    let _ = app
                                        .emit("estimated-time", json!({"seconds_remaining": secs}));
                                    let mut agent = agent_state.lock().await;
                                    agent.estimated_seconds = Some(secs as u32);
                                }
                            }
                        }
                    }
                }
                ContentBlock::ToolUse { id, name, input } => {
                    has_tool_use = true;
                    assistant_blocks.push(block.clone());

                    // Parse and execute the action
                    let mut action = match name.as_str() {
                        "computer" => {
                            // Scale factor 1.0: coords from AI are already in display_width x display_height space
                            parse_computer_action(input, 1.0)?
                        }
                        "bash" => {
                            let command = input["command"]
                                .as_str()
                                .or_else(|| input["input"].as_str())
                                .unwrap_or("")
                                .to_string();
                            AgentAction::BashCommand { command }
                        }
                        "text_editor" => {
                            let cmd = input["command"].as_str().unwrap_or("view");
                            let path = input["path"].as_str().unwrap_or("").to_string();
                            match cmd {
                                "view" => AgentAction::TextEditorView { path },
                                "create" => AgentAction::TextEditorCreate {
                                    path,
                                    content: input["file_text"].as_str().unwrap_or("").to_string(),
                                },
                                "str_replace" => AgentAction::TextEditorReplace {
                                    path,
                                    old_text: input["old_str"].as_str().unwrap_or("").to_string(),
                                    new_text: input["new_str"].as_str().unwrap_or("").to_string(),
                                },
                                _ => AgentAction::TextEditorView { path },
                            }
                        }
                        _ => {
                            tool_results.push(ContentBlock::ToolResult {
                                tool_use_id: id.clone(),
                                content: format!("Unknown tool: {}", name),
                                is_error: Some(true),
                            });
                            continue;
                        }
                    };

                    // Handle get_element_position early to prevent it from going to execute_action
                    if name.as_str() == "get_element_position" {
                        let elem_id = input["id"].as_i64().unwrap_or(-1) as i32;
                        let mut found = false;

                        if let Some(nodes) = &last_nodes {
                            if let Some(node) = nodes.iter().find(|n| n.id == elem_id) {
                                let (x, y, r, b) = node.rect;
                                let cx = x + (r - x) / 2;
                                let cy = y + (b - y) / 2;
                                tool_results.push(ContentBlock::ToolResult {
                                    tool_use_id: id.clone(),
                                    content: format!("Element ID {} position:\nBounding Box: [left: {}, top: {}, right: {}, bottom: {}]\nCenter: [cx: {}, cy: {}]", elem_id, x, y, r, b, cx, cy),
                                    is_error: None,
                                });
                                found = true;
                            }
                        }

                        if !found {
                            tool_results.push(ContentBlock::ToolResult {
                                tool_use_id: id.clone(),
                                content: format!("Error: Element [{}] not found in the current accessibility tree. Take a screenshot first.", elem_id),
                                is_error: Some(true),
                            });
                        }

                        // Delay and continue loop, bypassing standard execute_action
                        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        continue;
                    }

                    // Translate click_element by ID into LeftClick coordinates
                    if let AgentAction::ClickElement { id: elem_id } = action {
                        let mut found = false;
                        if let Some(nodes) = &last_nodes {
                            if let Some(node) = nodes.iter().find(|n| n.id == elem_id) {
                                let (x, y, r, b) = node.rect;
                                // Calculate center of bounding box
                                let cx = x + (r - x) / 2;
                                let cy = y + (b - y) / 2;
                                action = AgentAction::LeftClick { x: cx, y: cy };
                                found = true;
                            }
                        }
                        if !found {
                            tool_results.push(ContentBlock::ToolResult {
                                tool_use_id: id.clone(),
                                content: format!("Error: Element [{}] not found in the current accessibility tree. Take a screenshot first.", elem_id),
                                is_error: Some(true),
                            });
                            continue;
                        }
                    }

                    // Emit action executed
                    let _ = app.emit(
                        "action-executed",
                        json!({
                            "action_type": name,
                            "description": action.description(),
                            "timestamp": chrono::Utc::now().to_rfc3339(),
                            "iteration": iteration
                        }),
                    );

                    // Emit cursor position for mouse actions (moves the blue overlay cursor)
                    match &action {
                        AgentAction::LeftClick { x, y }
                        | AgentAction::RightClick { x, y }
                        | AgentAction::DoubleClick { x, y }
                        | AgentAction::MouseMove { x, y } => {
                            let _ = app.emit("cursor-moved", json!({"x": x, "y": y}));
                        }
                        _ => {}
                    }

                    // Emit click ripple for click actions
                    match &action {
                        AgentAction::LeftClick { x, y }
                        | AgentAction::RightClick { x, y }
                        | AgentAction::DoubleClick { x, y } => {
                            let _ = app.emit("cursor-click", json!({"x": x, "y": y}));
                        }
                        _ => {}
                    }

                    // Execute
                    let shell_timeout = settings.read().await.shell_timeout_secs;
                    let max_w = settings.read().await.display_width;
                    let max_h = settings.read().await.display_height;

                    match execute_action(&action, max_w, max_h, shell_timeout).await {
                        Ok(result) => {
                            if let Some(nodes) = result.nodes {
                                last_nodes = Some(nodes.clone());

                                // Format structural DOM
                                let mut dom_text = String::from(
                                    "Screenshot taken.\n\nAccessibility Tree (UI Elements):\n",
                                );
                                for n in nodes {
                                    dom_text.push_str(&format!(
                                        "[{}] {} \"{}\"\n",
                                        n.id, n.control_type, n.name
                                    ));
                                }

                                if let Some(base64) = result.base64 {
                                    let _ = app.emit(
                                        "screenshot-updated",
                                        json!({"base64": base64, "w": max_w, "h": max_h}),
                                    );
                                    tool_results.push(ContentBlock::ToolResult {
                                        tool_use_id: id.clone(),
                                        content: dom_text,
                                        is_error: None,
                                    });
                                    tool_results.push(ContentBlock::Image {
                                        source: ImageSource {
                                            source_type: "base64".to_string(),
                                            media_type: "image/jpeg".to_string(),
                                            data: base64,
                                        },
                                    });
                                }
                            } else {
                                tool_results.push(ContentBlock::ToolResult {
                                    tool_use_id: id.clone(),
                                    content: result.text,
                                    is_error: None,
                                });
                            }
                        }
                        Err(e) => {
                            tool_results.push(ContentBlock::ToolResult {
                                tool_use_id: id.clone(),
                                content: format!("Error: {}", e),
                                is_error: Some(true),
                            });
                        }
                    }

                    // Small delay between actions
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
                _ => {}
            }
        }

        // Add assistant message
        messages.push(Message {
            role: "assistant".to_string(),
            content: assistant_blocks,
        });

        // If there were tool results, add user message with results
        if !tool_results.is_empty() {
            messages.push(Message {
                role: "user".to_string(),
                content: tool_results,
            });
        }

        // Trim history
        trim_history(&mut messages);

        // If no tool use and stop reason is end_turn, the agent is done
        if !has_tool_use && response.stop_reason == "end_turn" {
            let _ = app.emit("agent-message", json!({"text": "Task completed."}));
            break;
        }

        iteration += 1;

        if cancel_token.is_cancelled() {
            hide_cursor_overlay(&app);
            return Ok(());
        }
    }

    hide_cursor_overlay(&app);
    Ok(())
}

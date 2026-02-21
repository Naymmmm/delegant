use serde::Serialize;

use crate::error::{AppError, AppResult};
use crate::input::{keyboard, mouse};
use crate::screen::capture::capture_screenshot;
use crate::shell::executor;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type")]
pub enum AgentAction {
    Screenshot,
    MouseMove {
        x: i32,
        y: i32,
    },
    LeftClick {
        x: i32,
        y: i32,
    },
    RightClick {
        x: i32,
        y: i32,
    },
    DoubleClick {
        x: i32,
        y: i32,
    },
    Type {
        text: String,
    },
    Key {
        combo: String,
    },
    Scroll {
        x: i32,
        y: i32,
        direction: String,
        amount: i32,
    },
    Wait {
        duration_ms: u64,
    },
    Drag {
        start_x: i32,
        start_y: i32,
        end_x: i32,
        end_y: i32,
    },
    BashCommand {
        command: String,
    },
    TextEditorView {
        path: String,
    },
    TextEditorCreate {
        path: String,
        content: String,
    },
    TextEditorReplace {
        path: String,
        old_text: String,
        new_text: String,
    },
    ClickElement {
        id: i32,
    },
}

impl AgentAction {
    pub fn description(&self) -> String {
        match self {
            Self::Screenshot => "Taking screenshot".to_string(),
            Self::MouseMove { x, y } => format!("Moving mouse to ({}, {})", x, y),
            Self::LeftClick { x, y } => format!("Left click at ({}, {})", x, y),
            Self::RightClick { x, y } => format!("Right click at ({}, {})", x, y),
            Self::DoubleClick { x, y } => format!("Double click at ({}, {})", x, y),
            Self::Type { text } => {
                let preview = if text.len() > 40 { &text[..40] } else { text };
                format!("Typing: \"{}\"", preview)
            }
            Self::Key { combo } => format!("Pressing: {}", combo),
            Self::Scroll {
                direction, amount, ..
            } => format!("Scrolling {} {} clicks", direction, amount),
            Self::Wait { duration_ms } => format!("Waiting {}ms", duration_ms),
            Self::Drag {
                start_x,
                start_y,
                end_x,
                end_y,
            } => {
                format!("Dragging ({},{}) → ({},{})", start_x, start_y, end_x, end_y)
            }
            Self::BashCommand { command } => {
                let preview = if command.len() > 60 {
                    &command[..60]
                } else {
                    command
                };
                format!("Running: {}", preview)
            }
            Self::TextEditorView { path } => format!("Viewing: {}", path),
            Self::TextEditorCreate { path, .. } => format!("Creating: {}", path),
            Self::TextEditorReplace { path, .. } => format!("Editing: {}", path),
            Self::ClickElement { id } => format!("Clicking element [{}]", id),
        }
    }
}

/// Parse a computer tool call from the AI response
pub fn parse_computer_action(
    input: &serde_json::Value,
    scale_factor: f64,
) -> AppResult<AgentAction> {
    let action = input["action"]
        .as_str()
        .ok_or_else(|| AppError::Agent("Missing action field".into()))?;

    let scale = |v: i32| -> i32 { (v as f64 / scale_factor) as i32 };

    match action {
        "screenshot" => Ok(AgentAction::Screenshot),
        "click_element" => {
            let id = input["id"]
                .as_i64()
                .ok_or_else(|| AppError::Agent("Missing id for click_element".into()))?
                as i32;
            Ok(AgentAction::ClickElement { id })
        }
        "mouse_move" => {
            let coords = parse_coords(input)?;
            Ok(AgentAction::MouseMove {
                x: scale(coords.0),
                y: scale(coords.1),
            })
        }
        "left_click" => {
            let coords = parse_coords(input)?;
            Ok(AgentAction::LeftClick {
                x: scale(coords.0),
                y: scale(coords.1),
            })
        }
        "right_click" => {
            let coords = parse_coords(input)?;
            Ok(AgentAction::RightClick {
                x: scale(coords.0),
                y: scale(coords.1),
            })
        }
        "double_click" => {
            let coords = parse_coords(input)?;
            Ok(AgentAction::DoubleClick {
                x: scale(coords.0),
                y: scale(coords.1),
            })
        }
        "type" => {
            let text = input["text"]
                .as_str()
                .ok_or_else(|| AppError::Agent("Missing text for type action".into()))?;
            Ok(AgentAction::Type {
                text: text.to_string(),
            })
        }
        "key" => {
            let text = input["text"]
                .as_str()
                .ok_or_else(|| AppError::Agent("Missing text for key action".into()))?;
            Ok(AgentAction::Key {
                combo: text.to_string(),
            })
        }
        "scroll" => {
            let coords = parse_coords(input)?;
            let direction = input["scroll_direction"]
                .as_str()
                .unwrap_or("down")
                .to_string();
            let amount = input["scroll_amount"].as_i64().unwrap_or(3) as i32;
            Ok(AgentAction::Scroll {
                x: scale(coords.0),
                y: scale(coords.1),
                direction,
                amount,
            })
        }
        "wait" => {
            let duration = input["duration"].as_u64().unwrap_or(1000);
            Ok(AgentAction::Wait {
                duration_ms: duration,
            })
        }
        "drag" => {
            let start = input["start_coordinate"]
                .as_array()
                .ok_or_else(|| AppError::Agent("Missing start_coordinate".into()))?;
            let end = input["end_coordinate"]
                .as_array()
                .ok_or_else(|| AppError::Agent("Missing end_coordinate".into()))?;
            Ok(AgentAction::Drag {
                start_x: scale(start[0].as_i64().unwrap_or(0) as i32),
                start_y: scale(start[1].as_i64().unwrap_or(0) as i32),
                end_x: scale(end[0].as_i64().unwrap_or(0) as i32),
                end_y: scale(end[1].as_i64().unwrap_or(0) as i32),
            })
        }
        _ => Err(AppError::Agent(format!(
            "Unknown computer action: {}",
            action
        ))),
    }
}

fn parse_coords(input: &serde_json::Value) -> AppResult<(i32, i32)> {
    let coords = input["coordinate"]
        .as_array()
        .ok_or_else(|| AppError::Agent("Missing coordinate field".into()))?;
    let x = coords.first().and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let y = coords.get(1).and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    Ok((x, y))
}

pub struct ActionResult {
    pub text: String,
    pub base64: Option<String>,
    pub nodes: Option<Vec<crate::screen::a11y::A11yNode>>,
}

/// Execute an agent action and return a result containing text, and optionally a screenshot and a11y nodes
pub async fn execute_action(
    action: &AgentAction,
    max_width: u32,
    max_height: u32,
    shell_timeout: u64,
) -> AppResult<ActionResult> {
    match action {
        AgentAction::Screenshot => {
            let result =
                tokio::task::spawn_blocking(move || capture_screenshot(max_width, max_height))
                    .await
                    .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Screenshot taken.".to_string(),
                base64: Some(result.base64),
                nodes: Some(result.nodes),
            })
        }
        AgentAction::MouseMove { x, y } => {
            let (x, y) = (*x, *y);
            tokio::task::spawn_blocking(move || mouse::move_to(x, y))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Mouse moved".to_string(),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::LeftClick { x, y } => {
            let (x, y) = (*x, *y);
            tokio::task::spawn_blocking(move || mouse::click(x, y, "left"))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Left clicked".to_string(),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::RightClick { x, y } => {
            let (x, y) = (*x, *y);
            tokio::task::spawn_blocking(move || mouse::click(x, y, "right"))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Right clicked".to_string(),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::DoubleClick { x, y } => {
            let (x, y) = (*x, *y);
            tokio::task::spawn_blocking(move || mouse::double_click(x, y))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Double clicked".to_string(),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::Type { text } => {
            let text = text.clone();
            tokio::task::spawn_blocking(move || keyboard::fast_type(&text))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Text typed".to_string(),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::Key { combo } => {
            let combo_clone = combo.clone();
            let combo_display = combo.clone();
            tokio::task::spawn_blocking(move || keyboard::press_key(&combo_clone))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: format!("Key pressed: {}", combo_display),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::Scroll {
            x,
            y,
            direction,
            amount,
        } => {
            let (x, y) = (*x, *y);
            let clicks = if direction == "up" { -*amount } else { *amount };
            tokio::task::spawn_blocking(move || mouse::scroll(x, y, clicks))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Scrolled".to_string(),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::Wait { duration_ms } => {
            tokio::time::sleep(std::time::Duration::from_millis(*duration_ms)).await;
            Ok(ActionResult {
                text: format!("Waited {}ms", duration_ms),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::Drag {
            start_x,
            start_y,
            end_x,
            end_y,
        } => {
            let (sx, sy, ex, ey) = (*start_x, *start_y, *end_x, *end_y);
            tokio::task::spawn_blocking(move || mouse::drag(sx, sy, ex, ey))
                .await
                .map_err(|e| AppError::Agent(e.to_string()))??;
            Ok(ActionResult {
                text: "Dragged".to_string(),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::BashCommand { command } => {
            let result = executor::run_command(command, shell_timeout).await?;
            let mut output = String::new();
            if !result.stdout.is_empty() {
                output.push_str(&result.stdout);
            }
            if !result.stderr.is_empty() {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str("STDERR: ");
                output.push_str(&result.stderr);
            }
            if output.is_empty() {
                output = format!("Command completed with exit code {}", result.exit_code);
            }
            Ok(ActionResult {
                text: output,
                base64: None,
                nodes: None,
            })
        }
        AgentAction::TextEditorView { path } => {
            match tokio::fs::read_to_string(path).await {
                Ok(content) => {
                    // Add line numbers
                    let numbered: String = content
                        .lines()
                        .enumerate()
                        .map(|(i, line)| format!("{:>4} | {}", i + 1, line))
                        .collect::<Vec<_>>()
                        .join("\n");
                    let truncated = if numbered.len() > 10000 {
                        format!("{}...[truncated]", &numbered[..10000])
                    } else {
                        numbered
                    };
                    Ok(ActionResult {
                        text: truncated,
                        base64: None,
                        nodes: None,
                    })
                }
                Err(e) => Ok(ActionResult {
                    text: format!("Error reading file: {}", e),
                    base64: None,
                    nodes: None,
                }),
            }
        }
        AgentAction::TextEditorCreate { path, content } => {
            if let Some(parent) = std::path::Path::new(path).parent() {
                tokio::fs::create_dir_all(parent).await.ok();
            }
            tokio::fs::write(path, content).await?;
            Ok(ActionResult {
                text: format!("File created: {}", path),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::TextEditorReplace {
            path,
            old_text,
            new_text,
        } => {
            let file_content = tokio::fs::read_to_string(path).await?;
            if !file_content.contains(old_text.as_str()) {
                return Ok(ActionResult {
                    text: format!("Error: old_str not found in {}", path),
                    base64: None,
                    nodes: None,
                });
            }
            let new_content = file_content.replacen(old_text.as_str(), new_text.as_str(), 1);
            tokio::fs::write(path, new_content).await?;
            Ok(ActionResult {
                text: format!("File edited: {}", path),
                base64: None,
                nodes: None,
            })
        }
        AgentAction::ClickElement { .. } => Ok(ActionResult {
            text: "Error: ClickElement should be translated to LeftClick before execution".into(),
            base64: None,
            nodes: None,
        }),
    }
}

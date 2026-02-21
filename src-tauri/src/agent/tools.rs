use serde_json::json;

use crate::ai::types::ToolDefinition;

pub fn build_tool_definitions(
    provider: &str,
    display_width: u32,
    display_height: u32,
) -> Vec<ToolDefinition> {
    match provider {
        "anthropic" => build_anthropic_tools(display_width, display_height),
        "openai" => build_openai_tools(display_width, display_height),
        "openrouter" => build_openai_tools(display_width, display_height),
        _ => build_anthropic_tools(display_width, display_height),
    }
}

fn build_anthropic_tools(display_width: u32, display_height: u32) -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "computer".to_string(),
            tool_type: Some("computer_20250124".to_string()),
            description: None,
            input_schema: None,
            display_width_px: Some(display_width),
            display_height_px: Some(display_height),
            display_number: Some(1),
        },
        ToolDefinition {
            name: "bash".to_string(),
            tool_type: Some("bash_20250124".to_string()),
            description: None,
            input_schema: None,
            display_width_px: None,
            display_height_px: None,
            display_number: None,
        },
        ToolDefinition {
            name: "text_editor".to_string(),
            tool_type: Some("text_editor_20250124".to_string()),
            description: None,
            input_schema: None,
            display_width_px: None,
            display_height_px: None,
            display_number: None,
        },
    ]
}

fn build_openai_tools(display_width: u32, display_height: u32) -> Vec<ToolDefinition> {
    let coord_desc = format!(
        "[x, y] pixel coordinates from top-left. Screen is {}x{} pixels. Click the center of the target element.",
        display_width, display_height
    );

    vec![
        ToolDefinition {
            name: "computer".to_string(),
            tool_type: None,
            description: Some(format!(
                "Control the computer. The screen is {}x{} pixels. Actions: screenshot (see screen), click_element (by ID), mouse_move, left_click, right_click, double_click, type, key, scroll, wait, drag. Always take a screenshot first. Coordinates are pixel [x, y]. Click the center of UI elements.",
                display_width, display_height
            )),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["screenshot", "click_element", "mouse_move", "left_click", "right_click", "double_click", "type", "key", "scroll", "wait", "drag"]
                    },
                    "id": {
                        "type": "integer",
                        "description": "Element ID to click (from the Accessibility Tree or screenshot)."
                    },
                    "coordinate": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": coord_desc
                    },
                    "text": {
                        "type": "string",
                        "description": "Text to type or key combo to press (e.g. 'Return', 'ctrl+c')"
                    },
                    "start_coordinate": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": format!("[x, y] start position for drag. Screen is {}x{} pixels.", display_width, display_height)
                    },
                    "end_coordinate": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": format!("[x, y] end position for drag. Screen is {}x{} pixels.", display_width, display_height)
                    },
                    "scroll_direction": {
                        "type": "string",
                        "enum": ["up", "down"],
                        "description": "Scroll direction"
                    },
                    "scroll_amount": {
                        "type": "integer",
                        "description": "Number of scroll clicks (default 3)"
                    },
                    "duration": {
                        "type": "integer",
                        "description": "Wait duration in milliseconds"
                    }
                },
                "required": ["action"]
            })),
            display_width_px: None,
            display_height_px: None,
            display_number: None,
        },
        ToolDefinition {
            name: "get_element_position".to_string(),
            tool_type: None,
            description: Some("Get the bounding box coordinates [left, top, right, bottom] and calculated center [cx, cy] of an element ID found in the Accessibility Tree.".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "id": {
                        "type": "integer",
                        "description": "The element ID from the Accessibility Tree"
                    }
                },
                "required": ["id"]
            })),
            display_width_px: None,
            display_height_px: None,
            display_number: None,
        },
        ToolDefinition {
            name: "bash".to_string(),
            tool_type: None,
            description: Some("Execute a shell command (Windows cmd/powershell) and return stdout/stderr. Use for file operations, installations, or when CLI is faster than GUI.".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "description": "The shell command to execute"
                    }
                },
                "required": ["command"]
            })),
            display_width_px: None,
            display_height_px: None,
            display_number: None,
        },
        ToolDefinition {
            name: "text_editor".to_string(),
            tool_type: None,
            description: Some("View or edit text files. Commands: view (read file), create (new file), str_replace (replace text).".to_string()),
            input_schema: Some(json!({
                "type": "object",
                "properties": {
                    "command": {
                        "type": "string",
                        "enum": ["view", "create", "str_replace", "insert", "undo_edit"]
                    },
                    "path": {
                        "type": "string",
                        "description": "Absolute file path"
                    },
                    "file_text": {
                        "type": "string",
                        "description": "Content for create command"
                    },
                    "old_str": {
                        "type": "string",
                        "description": "Exact string to find and replace"
                    },
                    "new_str": {
                        "type": "string",
                        "description": "Replacement string"
                    },
                    "insert_line": {
                        "type": "integer",
                        "description": "Line number for insert"
                    },
                    "new_str_insert": {
                        "type": "string",
                        "description": "Text to insert at line"
                    },
                    "view_range": {
                        "type": "array",
                        "items": {"type": "integer"},
                        "description": "[start_line, end_line] range to view"
                    }
                },
                "required": ["command", "path"]
            })),
            display_width_px: None,
            display_height_px: None,
            display_number: None,
        },
    ]
}

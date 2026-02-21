use crate::error::{AppError, AppResult};
use serde::Serialize;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub handle: isize,
    pub title: String,
}

pub fn enumerate_windows() -> AppResult<Vec<WindowInfo>> {
    // In macOS, enumerating all active windows reliably without accessibility permissions
    // or dropping to Objective-C/CoreGraphics is complex.
    // For now, we will use AppleScript as a quick cross-platform stub
    // to list visible windows of standard applications.
    // Note: This is computationally heavier but avoids needing a macos-specific crate at compile time.
    let script = r#"
        tell application "System Events"
            set windowList to {}
            set theApps to every application process whose visible is true
            repeat with theApp in theApps
                try
                    set appWindows to every window of theApp
                    repeat with theWindow in appWindows
                        set windowTitle to name of theWindow
                        if windowTitle is not "" then
                            set end of windowList to windowTitle
                        end if
                    end repeat
                end try
            end repeat
            return windowList
        end tell
    "#;

    let output = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
        .map_err(|e| AppError::Window(e.to_string()))?;

    if !output.status.success() {
        return Err(AppError::Window(format!(
            "AppleScript failed: {}",
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let result_str = String::from_utf8_lossy(&output.stdout);
    if result_str.trim().is_empty() {
        return Ok(Vec::new());
    }

    // output is comma-separated from AppleScript list
    let windows = result_str
        .split(", ")
        .enumerate()
        .map(|(i, title)| WindowInfo {
            handle: i as isize, // We don't have real handles from AppleScript list
            title: title.trim().to_string(),
        })
        .collect();

    Ok(windows)
}

pub fn focus_window(_handle: isize) -> AppResult<()> {
    // Focusing by handle is difficult when we don't have real handles.
    // We would need the application name to tell AppleScript to focus it.
    // This is a stub for now.
    Ok(())
}

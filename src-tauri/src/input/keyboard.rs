use enigo::{Direction, Enigo, Key, Keyboard, Settings};

use crate::error::{AppError, AppResult};

fn new_enigo() -> AppResult<Enigo> {
    Enigo::new(&Settings::default()).map_err(|e| AppError::Input(format!("Enigo init: {}", e)))
}

fn parse_key(key_str: &str) -> AppResult<Key> {
    match key_str.to_lowercase().as_str() {
        "return" | "enter" => Ok(Key::Return),
        "tab" => Ok(Key::Tab),
        "escape" | "esc" => Ok(Key::Escape),
        "backspace" => Ok(Key::Backspace),
        "delete" => Ok(Key::Delete),
        "space" => Ok(Key::Space),
        "up" => Ok(Key::UpArrow),
        "down" => Ok(Key::DownArrow),
        "left" => Ok(Key::LeftArrow),
        "right" => Ok(Key::RightArrow),
        "home" => Ok(Key::Home),
        "end" => Ok(Key::End),
        "pageup" | "page_up" => Ok(Key::PageUp),
        "pagedown" | "page_down" => Ok(Key::PageDown),
        "f1" => Ok(Key::F1),
        "f2" => Ok(Key::F2),
        "f3" => Ok(Key::F3),
        "f4" => Ok(Key::F4),
        "f5" => Ok(Key::F5),
        "f6" => Ok(Key::F6),
        "f7" => Ok(Key::F7),
        "f8" => Ok(Key::F8),
        "f9" => Ok(Key::F9),
        "f10" => Ok(Key::F10),
        "f11" => Ok(Key::F11),
        "f12" => Ok(Key::F12),
        "shift" => Ok(Key::Shift),
        "control" | "ctrl" => Ok(Key::Control),
        "alt" => Ok(Key::Alt),
        "meta" | "super" | "win" | "command" => Ok(Key::Meta),
        "capslock" | "caps_lock" => Ok(Key::CapsLock),
        s if s.len() == 1 => Ok(Key::Unicode(s.chars().next().unwrap())),
        _ => Err(AppError::Input(format!("Unknown key: {}", key_str))),
    }
}

/// Press a key combination like "ctrl+c", "alt+tab", or a single key like "enter"
pub fn press_key(combo: &str) -> AppResult<()> {
    let mut enigo = new_enigo()?;
    let parts: Vec<&str> = combo.split('+').map(|s| s.trim()).collect();

    if parts.len() == 1 {
        let key = parse_key(parts[0])?;
        enigo
            .key(key, Direction::Click)
            .map_err(|e| AppError::Input(e.to_string()))?;
    } else {
        let modifiers = &parts[..parts.len() - 1];
        let final_key = parse_key(parts[parts.len() - 1])?;

        for m in modifiers {
            let key = parse_key(m)?;
            enigo
                .key(key, Direction::Press)
                .map_err(|e| AppError::Input(e.to_string()))?;
        }

        enigo
            .key(final_key, Direction::Click)
            .map_err(|e| AppError::Input(e.to_string()))?;

        for m in modifiers.iter().rev() {
            let key = parse_key(m)?;
            enigo
                .key(key, Direction::Release)
                .map_err(|e| AppError::Input(e.to_string()))?;
        }
    }

    Ok(())
}

/// Type a string of text
pub fn type_string(text: &str) -> AppResult<()> {
    let mut enigo = new_enigo()?;
    enigo.text(text).map_err(|e| AppError::Input(e.to_string()))
}

/// Instantly type a string by placing it in the clipboard and pressing Ctrl+V
pub fn fast_type(text: &str) -> AppResult<()> {
    if let Ok(mut clipboard) = arboard::Clipboard::new() {
        // Try to save old text
        let old_text = clipboard.get_text().unwrap_or_default();

        // Set new text
        if clipboard.set_text(text).is_ok() {
            // Give the OS a tiny moment to process clipboard change
            std::thread::sleep(std::time::Duration::from_millis(10));

            // Press Ctrl+V
            let _ = press_key("ctrl+v");

            // Give the OS a moment to process the paste before restoring
            std::thread::sleep(std::time::Duration::from_millis(20));

            // Restore old text
            let _ = clipboard.set_text(old_text);
            return Ok(());
        }
    }

    // Fallback to slow typing
    type_string(text)
}

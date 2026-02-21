use crate::error::AppResult;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub handle: isize,
    pub title: String,
}

pub fn enumerate_windows() -> AppResult<Vec<WindowInfo>> {
    // Implementing X11 or Wayland window enumeration requires heavy dependencies.
    // For now we will return an empty list on Linux.
    Ok(Vec::new())
}

pub fn focus_window(_handle: isize) -> AppResult<()> {
    // Stub
    Ok(())
}

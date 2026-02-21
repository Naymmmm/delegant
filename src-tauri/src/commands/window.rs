use crate::error::AppResult;
use crate::platform::{enumerate_windows, focus_window as focus_win, WindowInfo};

#[tauri::command]
pub async fn list_windows() -> AppResult<Vec<WindowInfo>> {
    tokio::task::spawn_blocking(enumerate_windows)
        .await
        .map_err(|e| crate::error::AppError::Window(e.to_string()))?
}

#[tauri::command]
pub async fn focus_window(handle: isize) -> AppResult<()> {
    tokio::task::spawn_blocking(move || focus_win(handle))
        .await
        .map_err(|e| crate::error::AppError::Window(e.to_string()))?
}

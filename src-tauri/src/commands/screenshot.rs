use tauri::State;

use crate::error::AppResult;
use crate::screen::capture::{capture_screenshot, CaptureResult};
use crate::state::AppState;

#[tauri::command]
pub async fn take_screenshot(state: State<'_, AppState>) -> AppResult<CaptureResult> {
    let settings = state.settings.read().await;
    let max_w = settings.display_width;
    let max_h = settings.display_height;
    drop(settings);

    tokio::task::spawn_blocking(move || capture_screenshot(max_w, max_h))
        .await
        .map_err(|e| crate::error::AppError::Screenshot(e.to_string()))?
}

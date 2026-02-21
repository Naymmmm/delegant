use tauri::State;

use crate::error::AppResult;
use crate::state::{AppState, Settings};

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> AppResult<Settings> {
    let settings = state.settings.read().await;
    Ok(settings.clone())
}

#[tauri::command]
pub async fn save_settings(
    settings: Settings,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut current = state.settings.write().await;
    *current = settings;
    Ok(())
}

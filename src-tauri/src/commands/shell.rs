use tauri::State;

use crate::error::AppResult;
use crate::shell::executor::{run_command, ShellResult};
use crate::state::AppState;

#[tauri::command]
pub async fn run_shell(
    command: String,
    state: State<'_, AppState>,
) -> AppResult<ShellResult> {
    let timeout = state.settings.read().await.shell_timeout_secs;
    run_command(&command, timeout).await
}

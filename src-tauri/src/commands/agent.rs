use tauri::{AppHandle, Emitter, Manager, State};
use tokio_util::sync::CancellationToken;

use crate::agent::loop_runner;
use crate::error::{AppError, AppResult};
use crate::state::{AgentStatus, AppState};

fn restore_main_window(app: &AppHandle) {
    if let Some(cursor_window) = app.get_webview_window("cursor-overlay") {
        let _ = cursor_window.hide();
    }
}

#[tauri::command]
pub async fn start_agent(
    task: String,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    let mut agent = state.agent.lock().await;
    if agent.status == AgentStatus::Running {
        return Err(AppError::Agent("Agent is already running".into()));
    }

    let cancel_token = CancellationToken::new();
    agent.status = AgentStatus::Running;
    agent.current_task = Some(task.clone());
    agent.iteration = 0;
    agent.estimated_seconds = None;
    agent.cancel_token = Some(cancel_token.clone());
    drop(agent);

    let _ = app.emit("agent-status-changed", "running");

    // Show cursor overlay window (fullscreen, transparent, click-through)
    if let Some(cursor_window) = app.get_webview_window("cursor-overlay") {
        if let Some(window) = app.get_webview_window("main") {
            if let Ok(Some(monitor)) = window.primary_monitor() {
                let screen_size = monitor.size();
                let _ = cursor_window.set_size(tauri::Size::Physical(tauri::PhysicalSize::new(
                    screen_size.width,
                    screen_size.height,
                )));
                let _ = cursor_window.set_position(tauri::Position::Physical(
                    tauri::PhysicalPosition::new(0, 0),
                ));
            }
        }
        let _ = cursor_window.set_ignore_cursor_events(true);
        let _ = cursor_window.show();
    }

    // Spawn the agent loop
    let settings = state.settings.clone();
    let agent_state = state.agent.clone();
    let app_handle = app.clone();

    tokio::spawn(async move {
        let result = loop_runner::run_agent_loop(
            task,
            cancel_token,
            settings,
            agent_state.clone(),
            app_handle.clone(),
        )
        .await;

        let mut agent = agent_state.lock().await;
        match result {
            Ok(_) => {
                agent.status = AgentStatus::Idle;
                let _ = app_handle.emit("agent-status-changed", "idle");
            }
            Err(e) => {
                log::error!("Agent loop error: {}", e);
                agent.status = AgentStatus::Error;
                let _ = app_handle.emit("agent-status-changed", &format!("error:{}", e));
            }
        }
        agent.cancel_token = None;
        drop(agent);

        // Restore window to start screen size
        restore_main_window(&app_handle);
    });

    Ok(())
}

#[tauri::command]
pub async fn stop_agent(app: AppHandle, state: State<'_, AppState>) -> AppResult<()> {
    let mut agent = state.agent.lock().await;
    if let Some(token) = agent.cancel_token.take() {
        token.cancel();
    }
    agent.status = AgentStatus::Idle;
    agent.current_task = None;
    agent.iteration = 0;
    agent.estimated_seconds = None;
    drop(agent);

    let _ = app.emit("agent-status-changed", "idle");

    restore_main_window(&app);

    Ok(())
}

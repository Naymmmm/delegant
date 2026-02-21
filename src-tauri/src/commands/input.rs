use crate::error::AppResult;

#[tauri::command]
pub async fn mouse_move(x: i32, y: i32) -> AppResult<()> {
    tokio::task::spawn_blocking(move || crate::input::mouse::move_to(x, y))
        .await
        .map_err(|e| crate::error::AppError::Input(e.to_string()))?
}

#[tauri::command]
pub async fn mouse_click(x: i32, y: i32, button: String) -> AppResult<()> {
    tokio::task::spawn_blocking(move || crate::input::mouse::click(x, y, &button))
        .await
        .map_err(|e| crate::error::AppError::Input(e.to_string()))?
}

#[tauri::command]
pub async fn mouse_double_click(x: i32, y: i32) -> AppResult<()> {
    tokio::task::spawn_blocking(move || crate::input::mouse::double_click(x, y))
        .await
        .map_err(|e| crate::error::AppError::Input(e.to_string()))?
}

#[tauri::command]
pub async fn mouse_scroll(x: i32, y: i32, clicks: i32) -> AppResult<()> {
    tokio::task::spawn_blocking(move || crate::input::mouse::scroll(x, y, clicks))
        .await
        .map_err(|e| crate::error::AppError::Input(e.to_string()))?
}

#[tauri::command]
pub async fn mouse_drag(
    start_x: i32,
    start_y: i32,
    end_x: i32,
    end_y: i32,
) -> AppResult<()> {
    tokio::task::spawn_blocking(move || {
        crate::input::mouse::drag(start_x, start_y, end_x, end_y)
    })
    .await
    .map_err(|e| crate::error::AppError::Input(e.to_string()))?
}

#[tauri::command]
pub async fn key_press(combo: String) -> AppResult<()> {
    tokio::task::spawn_blocking(move || crate::input::keyboard::press_key(&combo))
        .await
        .map_err(|e| crate::error::AppError::Input(e.to_string()))?
}

#[tauri::command]
pub async fn type_text(text: String) -> AppResult<()> {
    tokio::task::spawn_blocking(move || crate::input::keyboard::type_string(&text))
        .await
        .map_err(|e| crate::error::AppError::Input(e.to_string()))?
}

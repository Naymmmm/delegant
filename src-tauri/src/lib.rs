mod agent;
mod ai;
mod commands;
mod error;
mod input;
mod platform;
mod screen;
mod shell;
mod state;

use state::AppState;
use tauri::{RunEvent, WindowEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Create cursor overlay window (hidden by default, fullscreen, click-through)
            let cursor_window = tauri::WebviewWindowBuilder::new(
                app,
                "cursor-overlay",
                tauri::WebviewUrl::App("cursor.html".into()),
            )
            .title("Cursor Overlay")
            .decorations(false)
            .transparent(true)
            .always_on_top(true)
            .skip_taskbar(true)
            .visible(false)
            .resizable(false)
            .focused(false)
            .shadow(false)
            .build()?;

            // Set cursor window to full primary monitor size
            if let Ok(Some(monitor)) = cursor_window.primary_monitor() {
                let size = monitor.size();
                let _ = cursor_window.set_size(tauri::PhysicalSize::new(size.width, size.height));
                let _ = cursor_window.set_position(tauri::PhysicalPosition::new(0, 0));
            }

            // Make cursor window click-through using Windows API
            #[cfg(target_os = "windows")]
            {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{
                    GetWindowLongW, SetWindowLongW, GWL_EXSTYLE, WS_EX_LAYERED, WS_EX_TRANSPARENT,
                };

                if let Ok(hwnd) = cursor_window.hwnd() {
                    unsafe {
                        let ex_style = GetWindowLongW(HWND(hwnd.0), GWL_EXSTYLE);
                        SetWindowLongW(
                            HWND(hwnd.0),
                            GWL_EXSTYLE,
                            ex_style | WS_EX_LAYERED.0 as i32 | WS_EX_TRANSPARENT.0 as i32,
                        );
                    }
                }
            }

            Ok(())
        })
        .manage(AppState::new())
        .invoke_handler(tauri::generate_handler![
            commands::screenshot::take_screenshot,
            commands::input::mouse_move,
            commands::input::mouse_click,
            commands::input::mouse_double_click,
            commands::input::mouse_scroll,
            commands::input::mouse_drag,
            commands::input::key_press,
            commands::input::type_text,
            commands::shell::run_shell,
            commands::window::list_windows,
            commands::window::focus_window,
            commands::agent::start_agent,
            commands::agent::stop_agent,
            commands::settings::get_settings,
            commands::settings::save_settings,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|app, event| {
            if let RunEvent::WindowEvent {
                label,
                event: WindowEvent::CloseRequested { .. },
                ..
            } = &event
            {
                if label == "main" {
                    app.exit(0);
                }
            }
        });
}

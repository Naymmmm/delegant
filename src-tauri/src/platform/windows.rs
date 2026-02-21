use crate::error::{AppError, AppResult};
use serde::Serialize;
use windows::Win32::Foundation::{BOOL, HWND, LPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetWindowTextLengthW, GetWindowTextW, IsWindowVisible, SetForegroundWindow,
    ShowWindow, SW_RESTORE,
};

#[derive(Debug, Clone, Serialize)]
pub struct WindowInfo {
    pub handle: isize,
    pub title: String,
}

unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    let windows = &mut *(lparam.0 as *mut Vec<WindowInfo>);

    if !IsWindowVisible(hwnd).as_bool() {
        return BOOL(1);
    }

    let length = GetWindowTextLengthW(hwnd);
    if length == 0 {
        return BOOL(1);
    }

    let mut buf = vec![0u16; (length + 1) as usize];
    GetWindowTextW(hwnd, &mut buf);
    let title = String::from_utf16_lossy(&buf[..length as usize]);

    if !title.is_empty() {
        windows.push(WindowInfo {
            handle: hwnd.0 as isize,
            title,
        });
    }

    BOOL(1)
}

pub fn enumerate_windows() -> AppResult<Vec<WindowInfo>> {
    let mut windows: Vec<WindowInfo> = Vec::new();
    unsafe {
        EnumWindows(
            Some(enum_windows_proc),
            LPARAM(&mut windows as *mut Vec<WindowInfo> as isize),
        )
        .map_err(|e| AppError::Window(e.to_string()))?;
    }
    Ok(windows)
}

pub fn focus_window(handle: isize) -> AppResult<()> {
    unsafe {
        let hwnd = HWND(handle as *mut _);
        let _ = ShowWindow(hwnd, SW_RESTORE);
        let result = SetForegroundWindow(hwnd);
        if !result.as_bool() {
            return Err(AppError::Window("Failed to focus window".to_string()));
        }
    }
    Ok(())
}

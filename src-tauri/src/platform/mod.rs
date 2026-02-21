#[cfg(target_os = "windows")]
pub mod windows;
#[cfg(target_os = "windows")]
pub use windows::{enumerate_windows, focus_window, WindowInfo};

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "macos")]
pub use macos::{enumerate_windows, focus_window, WindowInfo};

#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "linux")]
pub use linux::{enumerate_windows, focus_window, WindowInfo};

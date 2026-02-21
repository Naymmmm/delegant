use serde::Serialize;

#[derive(Debug, thiserror::Error)]
#[allow(dead_code)]
pub enum AppError {
    #[error("Screenshot failed: {0}")]
    Screenshot(String),

    #[error("Input error: {0}")]
    Input(String),

    #[error("Shell error: {0}")]
    Shell(String),

    #[error("AI provider error: {0}")]
    AiProvider(String),

    #[error("Agent error: {0}")]
    Agent(String),

    #[error("Window error: {0}")]
    Window(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("Tauri error: {0}")]
    Tauri(#[from] tauri::Error),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;

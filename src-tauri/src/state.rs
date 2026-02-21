use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AgentStatus {
    Idle,
    Running,
    Paused,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub anthropic_api_key: String,
    pub openai_api_key: String,
    pub openrouter_api_key: String,
    pub provider: String,
    pub model: String,
    pub display_width: u32,
    pub display_height: u32,
    pub shell_timeout_secs: u64,
    pub setup_complete: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            anthropic_api_key: String::new(),
            openai_api_key: String::new(),
            openrouter_api_key: String::new(),
            provider: "anthropic".to_string(),
            model: "claude-sonnet-4-20250514".to_string(),
            display_width: 1280,
            display_height: 800,
            shell_timeout_secs: 30,
            setup_complete: false,
        }
    }
}

#[derive(Debug)]
pub struct AgentState {
    pub status: AgentStatus,
    pub current_task: Option<String>,
    pub iteration: u32,
    pub estimated_seconds: Option<u32>,
    pub cancel_token: Option<CancellationToken>,
}

impl Default for AgentState {
    fn default() -> Self {
        Self {
            status: AgentStatus::Idle,
            current_task: None,
            iteration: 0,
            estimated_seconds: None,
            cancel_token: None,
        }
    }
}

pub struct AppState {
    pub agent: Arc<Mutex<AgentState>>,
    pub settings: Arc<RwLock<Settings>>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            agent: Arc::new(Mutex::new(AgentState::default())),
            settings: Arc::new(RwLock::new(Settings::default())),
        }
    }
}

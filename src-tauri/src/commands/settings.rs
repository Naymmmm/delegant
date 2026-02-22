use reqwest::Client;
use serde::Serialize;
use serde_json::Value;
use tauri::State;

use crate::error::{AppError, AppResult};
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

#[derive(Debug, Clone, Serialize)]
pub struct ModelOption {
    pub id: String,
    pub label: String,
}

#[tauri::command]
pub async fn list_ollama_models(
    base_url: String,
    api_key: Option<String>,
) -> AppResult<Vec<ModelOption>> {
    let normalized = normalize_ollama_base_url(&base_url);
    let mut req = Client::new().get(format!("{}/api/tags", normalized));

    if let Some(key) = api_key {
        let trimmed = key.trim();
        if !trimmed.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", trimmed));
        }
    }

    let resp = req.send().await?;
    let status = resp.status();
    if !status.is_success() {
        return Err(AppError::Settings(format!(
            "Ollama model fetch failed: HTTP {}",
            status
        )));
    }

    let parsed: Value = resp.json().await?;
    let mut models = parsed["models"]
        .as_array()
        .cloned()
        .unwrap_or_default();

    models.sort_by(|a, b| {
        b["modified_at"]
            .as_str()
            .unwrap_or("")
            .cmp(a["modified_at"].as_str().unwrap_or(""))
    });

    let out: Vec<ModelOption> = models
        .iter()
        .filter_map(|m| {
            let id = m["model"].as_str().or_else(|| m["name"].as_str())?;
            let label = m["name"].as_str().unwrap_or(id);
            Some(ModelOption {
                id: id.to_string(),
                label: label.to_string(),
            })
        })
        .collect();

    Ok(out)
}

fn normalize_ollama_base_url(input: &str) -> String {
    let trimmed = input.trim();
    let mut url = if trimmed.is_empty() {
        "http://127.0.0.1:11434".to_string()
    } else if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
        trimmed.to_string()
    } else {
        format!("http://{}", trimmed)
    };

    while url.ends_with('/') {
        url.pop();
    }

    if url.ends_with("/v1") {
        url.truncate(url.len() - 3);
    }

    while url.ends_with('/') {
        url.pop();
    }

    url
}

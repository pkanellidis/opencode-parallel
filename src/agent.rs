use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub id: String,
    pub provider: String,
    pub model: String,
    pub task: String,
    pub status: AgentStatus,
    pub output: Vec<String>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

impl AgentConfig {
    pub fn new(provider: &str, model: &str, task: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            provider: provider.to_string(),
            model: model.to_string(),
            task: task.to_string(),
            status: AgentStatus::Pending,
            output: Vec::new(),
            started_at: None,
            completed_at: None,
        }
    }

    pub fn start(&mut self) {
        self.status = AgentStatus::Running;
        self.started_at = Some(chrono::Utc::now());
    }

    pub fn complete(&mut self) {
        self.status = AgentStatus::Completed;
        self.completed_at = Some(chrono::Utc::now());
    }

    pub fn fail(&mut self) {
        self.status = AgentStatus::Failed;
        self.completed_at = Some(chrono::Utc::now());
    }

    pub fn cancel(&mut self) {
        self.status = AgentStatus::Cancelled;
        self.completed_at = Some(chrono::Utc::now());
    }

    pub fn add_output(&mut self, line: String) {
        self.output.push(line);
    }

    pub fn duration(&self) -> Option<chrono::Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some(end - start),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthConfig {
    providers: HashMap<String, String>,
}

fn get_auth_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home)
        .join(".config")
        .join("opencode-parallel")
        .join("auth.json")
}

pub fn configure_auth(provider: &str, key: Option<&str>) -> Result<()> {
    let auth_path = get_auth_path();
    
    if let Some(parent) = auth_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut config: AuthConfig = if auth_path.exists() {
        let content = fs::read_to_string(&auth_path)?;
        serde_json::from_str(&content)?
    } else {
        AuthConfig {
            providers: HashMap::new(),
        }
    };

    if let Some(api_key) = key {
        config.providers.insert(provider.to_string(), api_key.to_string());
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&auth_path, content)?;
        println!("✓ Configured {} provider", provider);
    } else {
        println!("Enter API key for {}: ", provider);
        let mut api_key = String::new();
        std::io::stdin().read_line(&mut api_key)?;
        let api_key = api_key.trim();
        config.providers.insert(provider.to_string(), api_key.to_string());
        let content = serde_json::to_string_pretty(&config)?;
        fs::write(&auth_path, content)?;
        println!("✓ Configured {} provider", provider);
    }

    Ok(())
}

pub fn list_providers() -> Result<()> {
    let auth_path = get_auth_path();
    
    if !auth_path.exists() {
        println!("No providers configured yet.");
        println!("Use 'opencode-parallel auth <provider>' to configure a provider.");
        return Ok(());
    }

    let content = fs::read_to_string(&auth_path)?;
    let config: AuthConfig = serde_json::from_str(&content)?;

    println!("Configured providers:");
    for (provider, _) in config.providers.iter() {
        println!("  • {}", provider);
    }

    Ok(())
}

pub fn get_provider_key(provider: &str) -> Result<String> {
    let auth_path = get_auth_path();
    
    if !auth_path.exists() {
        anyhow::bail!("No authentication configured. Run 'opencode-parallel auth {}' first.", provider);
    }

    let content = fs::read_to_string(&auth_path)?;
    let config: AuthConfig = serde_json::from_str(&content)?;

    config
        .providers
        .get(provider)
        .cloned()
        .context(format!("Provider '{}' not configured", provider))
}

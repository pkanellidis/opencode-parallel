use serde::{Deserialize, Serialize};
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

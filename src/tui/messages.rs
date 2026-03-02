//! Internal message types for TUI async communication.

use crate::orchestrator::TaskPlan;
use crate::server::StreamEvent;

/// A model option for the model selector.
#[derive(Clone, Debug)]
pub struct ModelOption {
    pub provider_id: String,
    pub provider_name: String,
    pub model_id: String,
    pub model_name: String,
}

impl ModelOption {
    /// Returns a display string for the model.
    pub fn display(&self) -> String {
        let provider_display = if self.provider_name != self.provider_id {
            &self.provider_name
        } else {
            &self.provider_id
        };

        if self.model_name != self.model_id {
            format!(
                "{}: {} ({})",
                provider_display, self.model_name, self.model_id
            )
        } else {
            format!("{}: {}", provider_display, self.model_id)
        }
    }
}

/// A pending permission request.
#[derive(Clone, Debug)]
pub struct PendingPermission {
    /// Request ID from the server.
    pub request_id: String,
    /// Session ID this permission is for.
    #[allow(dead_code)]
    pub session_id: String,
    /// The permission being requested (e.g., "edit", "bash").
    pub permission: String,
    /// File patterns affected.
    pub patterns: Vec<String>,
    /// Worker ID if this is for a worker.
    pub worker_id: Option<u32>,
    /// Worker description if this is for a worker.
    pub worker_description: Option<String>,
}

/// Messages sent between async tasks and the main TUI loop.
pub enum AppMessage {
    /// Log message from the orchestrator.
    OrchestratorLog(usize, String),
    /// Server logs updated.
    ServerLogs(Vec<String>),
    /// Task plan received from orchestrator.
    TaskPlan(usize, TaskPlan, Vec<String>, String),
    /// A worker has started.
    WorkerStarted(usize, u32, String),
    /// Worker produced output.
    WorkerOutput(usize, u32, String),
    /// Worker completed successfully.
    WorkerComplete(usize, u32),
    /// Worker encountered an error.
    WorkerError(usize, u32, String),
    /// Server-sent event received.
    StreamEvent(StreamEvent),
    /// Command completed with result message.
    CommandResult(String),
    /// An error occurred.
    Error(String),
    /// Models loaded for selector.
    ModelsLoaded(Vec<ModelOption>),
    /// Report worker results to orchestrator.
    ReportToOrchestrator(usize, String),
    /// Current model info loaded.
    CurrentModelLoaded(Option<String>),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn model_option_display_with_different_names() {
        let opt = ModelOption {
            provider_id: "openai".to_string(),
            provider_name: "OpenAI".to_string(),
            model_id: "gpt-4".to_string(),
            model_name: "GPT-4".to_string(),
        };
        assert_eq!(opt.display(), "OpenAI: GPT-4 (gpt-4)");
    }

    #[test]
    fn model_option_display_with_same_names() {
        let opt = ModelOption {
            provider_id: "openai".to_string(),
            provider_name: "openai".to_string(),
            model_id: "gpt-4".to_string(),
            model_name: "gpt-4".to_string(),
        };
        assert_eq!(opt.display(), "openai: gpt-4");
    }
}

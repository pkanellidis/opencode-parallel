//! Data types for the OpenCode server API.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Response from the health check endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
}

/// A session in the OpenCode server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
}

/// A message in a session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub role: String,
}

/// A part of a message (text, tool call, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

/// Response containing a message and its parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub info: Message,
    pub parts: Vec<Part>,
}

/// A project in the OpenCode workspace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub worktree: String,
    #[serde(default)]
    pub vcs: Option<String>,
}

/// Path information response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResponse {
    pub path: String,
}

/// A model available from a provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

/// An AI provider with its available models.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub models: HashMap<String, Model>,
}

/// Response from the providers endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub all: Vec<Provider>,
    #[serde(default)]
    pub default: serde_json::Value,
    #[serde(default)]
    pub connected: Vec<String>,
}

/// Information about a question from the question tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInfo {
    pub question: String,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub options: Vec<QuestionOption>,
}

/// An option in a question.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

/// Request to create a new session.
#[derive(Debug, Serialize)]
pub(crate) struct CreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
}

/// Request to send a message.
#[derive(Debug, Serialize)]
pub(crate) struct SendMessageRequest {
    pub parts: Vec<MessagePart>,
}

/// A part of a message being sent.
#[derive(Debug, Serialize)]
pub(crate) struct MessagePart {
    #[serde(rename = "type")]
    pub part_type: String,
    pub text: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_health_response() {
        let json = r#"{"healthy": true, "version": "1.0.0"}"#;
        let resp: HealthResponse = serde_json::from_str(json).unwrap();
        assert!(resp.healthy);
        assert_eq!(resp.version, "1.0.0");
    }

    #[test]
    fn deserialize_session() {
        let json = r#"{"id": "ses_123", "title": "My Session", "parentID": null}"#;
        let session: Session = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, "ses_123");
        assert_eq!(session.title, Some("My Session".to_string()));
        assert!(session.parent_id.is_none());
    }

    #[test]
    fn deserialize_session_minimal() {
        let json = r#"{"id": "ses_123"}"#;
        let session: Session = serde_json::from_str(json).unwrap();
        assert_eq!(session.id, "ses_123");
        assert!(session.title.is_none());
    }

    #[test]
    fn deserialize_message_response() {
        let json = r#"{
            "info": {"id": "msg_1", "sessionID": "ses_1", "role": "assistant"},
            "parts": [{"id": "p_1", "type": "text", "text": "Hello!"}]
        }"#;
        let resp: MessageResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.info.role, "assistant");
        assert_eq!(resp.parts.len(), 1);
        assert_eq!(resp.parts[0].text, Some("Hello!".to_string()));
    }

    #[test]
    fn deserialize_provider_response() {
        let json = r#"{
            "all": [{"id": "openai", "name": "OpenAI", "models": {}}],
            "default": {},
            "connected": ["openai"]
        }"#;
        let resp: ProviderResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.all.len(), 1);
        assert_eq!(resp.connected, vec!["openai"]);
    }

    #[test]
    fn deserialize_question_info() {
        let json = r#"{
            "question": "What file?",
            "header": "File Selection",
            "options": [{"label": "foo.txt", "description": "A text file"}]
        }"#;
        let info: QuestionInfo = serde_json::from_str(json).unwrap();
        assert_eq!(info.question, "What file?");
        assert_eq!(info.options.len(), 1);
        assert_eq!(info.options[0].label, "foo.txt");
    }
}

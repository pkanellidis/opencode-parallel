//! Server-sent events (SSE) handling for real-time updates.

use super::types::{Part, QuestionInfo};

/// Events received from the OpenCode server SSE stream.
#[derive(Debug, Clone)]
pub enum StreamEvent {
    /// Successfully connected to the event stream.
    Connected,

    /// A message part was updated (e.g., streaming text).
    PartUpdated { session_id: String, part: Part },

    /// A tool was called by the assistant.
    ToolCall {
        session_id: String,
        tool_name: String,
        status: String,
        input: serde_json::Value,
    },

    /// A session became idle (finished processing).
    SessionIdle { session_id: String },

    /// The assistant is asking a question via the question tool.
    QuestionAsked {
        session_id: String,
        request_id: String,
        questions: Vec<QuestionInfo>,
    },

    /// A permission is being requested for a tool operation.
    PermissionAsked {
        session_id: String,
        request_id: String,
        permission: String,
        patterns: Vec<String>,
    },

    /// An error occurred in the event stream.
    Error(String),
}

impl StreamEvent {
    /// Returns the session ID associated with this event, if any.
    pub fn session_id(&self) -> Option<&str> {
        match self {
            StreamEvent::PartUpdated { session_id, .. } => Some(session_id),
            StreamEvent::ToolCall { session_id, .. } => Some(session_id),
            StreamEvent::SessionIdle { session_id } => Some(session_id),
            StreamEvent::QuestionAsked { session_id, .. } => Some(session_id),
            StreamEvent::PermissionAsked { session_id, .. } => Some(session_id),
            StreamEvent::Connected | StreamEvent::Error(_) => None,
        }
    }

    /// Returns true if this is an error event.
    pub fn is_error(&self) -> bool {
        matches!(self, StreamEvent::Error(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_id_returns_correct_value() {
        let event = StreamEvent::SessionIdle {
            session_id: "ses_123".to_string(),
        };
        assert_eq!(event.session_id(), Some("ses_123"));
    }

    #[test]
    fn session_id_returns_none_for_connected() {
        let event = StreamEvent::Connected;
        assert_eq!(event.session_id(), None);
    }

    #[test]
    fn session_id_returns_none_for_error() {
        let event = StreamEvent::Error("test error".to_string());
        assert_eq!(event.session_id(), None);
    }

    #[test]
    fn is_error_returns_true_for_error() {
        let event = StreamEvent::Error("test".to_string());
        assert!(event.is_error());
    }

    #[test]
    fn is_error_returns_false_for_other_events() {
        let event = StreamEvent::Connected;
        assert!(!event.is_error());
    }
}

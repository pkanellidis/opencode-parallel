//! HTTP client for the OpenCode server API.

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

use super::events::StreamEvent;
use super::logs::ServerLogs;
use super::types::*;
use crate::utils::truncate_str;

const DEFAULT_HOST: &str = "127.0.0.1";

/// Client for interacting with the OpenCode server API.
///
/// Provides methods for session management, messaging, and configuration.
#[derive(Debug, Clone)]
pub struct OpenCodeServer {
    client: Client,
    base_url: String,
    pub logs: ServerLogs,
}

impl OpenCodeServer {
    /// Creates a new server client connecting to the given port.
    pub fn new(port: u16) -> Self {
        let logs = ServerLogs::new();
        logs.log(format!("Creating OpenCodeServer for port {}", port));
        Self {
            client: Client::new(),
            base_url: format!("http://{}:{}", DEFAULT_HOST, port),
            logs,
        }
    }

    /// Returns the base URL for the server.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    // -------------------------------------------------------------------------
    // Health & Status
    // -------------------------------------------------------------------------

    /// Checks if the server is healthy and returns version info.
    pub async fn health(&self) -> Result<HealthResponse> {
        self.logs.log("GET /global/health");
        let resp: HealthResponse = self
            .client
            .get(format!("{}/global/health", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        self.logs.log(format!(
            "Health response: healthy={}, version={}",
            resp.healthy, resp.version
        ));
        Ok(resp)
    }

    /// Returns true if the server is healthy.
    pub async fn is_healthy(&self) -> bool {
        self.health().await.map(|h| h.healthy).unwrap_or(false)
    }

    // -------------------------------------------------------------------------
    // Session Management
    // -------------------------------------------------------------------------

    /// Creates a new session with an optional title.
    pub async fn create_session(&self, title: Option<&str>) -> Result<Session> {
        self.logs.log(format!("POST /session (title={:?})", title));
        let req = CreateSessionRequest {
            title: title.map(String::from),
        };

        let resp = self
            .client
            .post(format!("{}/session", self.base_url))
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));

        let body = resp.text().await?;
        self.logs
            .log(format!("Response body: {}", truncate_str(&body, 200)));

        let session: Session = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse session: {}. Body: {}",
                e,
                truncate_str(&body, 500)
            )
        })?;
        self.logs.log(format!("Session created: id={}", session.id));
        Ok(session)
    }

    // -------------------------------------------------------------------------
    // Messaging
    // -------------------------------------------------------------------------

    /// Sends a message and waits for the complete response (blocking).
    pub async fn send_message(&self, session_id: &str, text: &str) -> Result<MessageResponse> {
        self.logs
            .log(format!("POST /session/{}/message", session_id));
        self.logs
            .log(format!("Message text: {}", truncate_str(text, 100)));

        let req = SendMessageRequest {
            parts: vec![MessagePart {
                part_type: "text".to_string(),
                text: text.to_string(),
            }],
        };

        let resp = self
            .client
            .post(format!("{}/session/{}/message", self.base_url, session_id))
            .json(&req)
            .send()
            .await?;

        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));

        let body = resp.text().await?;
        self.logs
            .log(format!("Response length: {} bytes", body.len()));

        let msg_response: MessageResponse = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse message response: {}. Body: {}",
                e,
                truncate_str(&body, 1000)
            )
        })?;

        self.logs.log(format!(
            "Message response: {} parts",
            msg_response.parts.len()
        ));
        Ok(msg_response)
    }

    /// Sends a message asynchronously (returns immediately, response via SSE).
    pub async fn send_message_async(&self, session_id: &str, text: &str) -> Result<()> {
        self.logs
            .log(format!("POST /session/{}/prompt_async", session_id));
        self.logs
            .log(format!("Message text: {}", truncate_str(text, 100)));

        let req = SendMessageRequest {
            parts: vec![MessagePart {
                part_type: "text".to_string(),
                text: text.to_string(),
            }],
        };

        let response = self
            .client
            .post(format!(
                "{}/session/{}/prompt_async",
                self.base_url, session_id
            ))
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        self.logs.log(format!("Response status: {}", status));

        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Async message failed: {} - {}",
                status,
                body
            ))
        }
    }

    // -------------------------------------------------------------------------
    // Projects & Paths
    // -------------------------------------------------------------------------

    /// Lists all available projects.
    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        self.logs.log("GET /project");
        let resp = self
            .client
            .get(format!("{}/project", self.base_url))
            .send()
            .await?;

        let body = resp.text().await?;
        let projects: Vec<Project> = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse projects: {}. Body: {}",
                e,
                truncate_str(&body, 500)
            )
        })?;
        Ok(projects)
    }

    /// Gets the current project.
    pub async fn get_current_project(&self) -> Result<Project> {
        self.logs.log("GET /project/current");
        let resp = self
            .client
            .get(format!("{}/project/current", self.base_url))
            .send()
            .await?;

        let body = resp.text().await?;
        let project: Project = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse project: {}. Body: {}",
                e,
                truncate_str(&body, 500)
            )
        })?;
        Ok(project)
    }

    /// Gets the current working path.
    pub async fn get_path(&self) -> Result<String> {
        self.logs.log("GET /path");
        let resp = self
            .client
            .get(format!("{}/path", self.base_url))
            .send()
            .await?;

        let body = resp.text().await?;

        // Try different response formats
        if let Ok(path_resp) = serde_json::from_str::<PathResponse>(&body) {
            return Ok(path_resp.path);
        }
        if let Ok(path) = serde_json::from_str::<String>(&body) {
            return Ok(path);
        }
        Ok(body.trim_matches('"').to_string())
    }

    // -------------------------------------------------------------------------
    // Configuration & Models
    // -------------------------------------------------------------------------

    /// Gets all available providers and their models.
    pub async fn get_providers(&self) -> Result<ProviderResponse> {
        self.logs.log("GET /provider");
        let resp = self
            .client
            .get(format!("{}/provider", self.base_url))
            .send()
            .await?;

        let body = resp.text().await?;
        let providers: ProviderResponse = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse providers: {}. Body: {}",
                e,
                truncate_str(&body, 500)
            )
        })?;
        Ok(providers)
    }

    /// Gets the current configuration.
    pub async fn get_config(&self) -> Result<serde_json::Value> {
        self.logs.log("GET /config");
        let resp = self
            .client
            .get(format!("{}/config", self.base_url))
            .send()
            .await?;

        let body = resp.text().await?;
        let config: serde_json::Value = serde_json::from_str(&body).map_err(|e| {
            anyhow::anyhow!(
                "Failed to parse config: {}. Body: {}",
                e,
                truncate_str(&body, 500)
            )
        })?;
        Ok(config)
    }

    /// Sets the current model.
    pub async fn set_model(&self, provider_id: &str, model_id: &str) -> Result<()> {
        self.logs.log(format!(
            "PATCH /config provider={} model={}",
            provider_id, model_id
        ));

        let body = serde_json::json!({
            "provider": provider_id,
            "model": model_id
        });

        let resp = self
            .client
            .patch(format!("{}/config", self.base_url))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to set model: {} - {}",
                status,
                body
            ))
        }
    }

    // -------------------------------------------------------------------------
    // Questions & Permissions
    // -------------------------------------------------------------------------

    /// Replies to a question tool request.
    pub async fn reply_to_question(
        &self,
        request_id: &str,
        answers: Vec<Vec<String>>,
    ) -> Result<()> {
        self.logs
            .log(format!("POST /question/{}/reply", request_id));

        let body = serde_json::json!({ "answers": answers });

        let resp = self
            .client
            .post(format!("{}/question/{}/reply", self.base_url, request_id))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to reply to question: {} - {}",
                status,
                body
            ))
        }
    }

    /// Replies to a permission request.
    ///
    /// Valid replies: "once", "always", "reject"
    pub async fn reply_to_permission(&self, request_id: &str, reply: &str) -> Result<()> {
        self.logs.log(format!(
            "POST /permission/{}/reply reply={}",
            request_id, reply
        ));

        let body = serde_json::json!({ "reply": reply });

        let resp = self
            .client
            .post(format!("{}/permission/{}/reply", self.base_url, request_id))
            .json(&body)
            .send()
            .await?;

        let status = resp.status();
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to reply to permission: {} - {}",
                status,
                body
            ))
        }
    }

    // -------------------------------------------------------------------------
    // Event Subscription
    // -------------------------------------------------------------------------

    /// Subscribes to server-sent events.
    ///
    /// Events are sent to the provided channel. This spawns a background task
    /// that runs until the channel is dropped.
    pub fn subscribe_events(&self, tx: mpsc::Sender<StreamEvent>) {
        let url = format!("{}/event", self.base_url);
        let logs = self.logs.clone();

        logs.log(format!("Subscribing to SSE: {}", url));

        tokio::spawn(async move {
            let mut es = EventSource::get(&url);

            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => {
                        logs.log("SSE connection opened");
                        let _ = tx.send(StreamEvent::Connected).await;
                    }
                    Ok(Event::Message(msg)) => {
                        if let Some(stream_event) = parse_sse_message(&msg.data, &logs) {
                            if tx.send(stream_event).await.is_err() {
                                logs.log("SSE receiver dropped, closing connection");
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        logs.log(format!("SSE error: {}", e));
                        let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                    }
                }
            }

            logs.log("SSE connection closed");
        });
    }
}

/// Parses an SSE message into a StreamEvent.
fn parse_sse_message(data: &str, logs: &ServerLogs) -> Option<StreamEvent> {
    let parsed: serde_json::Value = serde_json::from_str(data).ok()?;

    let event_type = parsed
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("unknown");

    logs.log(format!("[SSE] {}", event_type));

    match event_type {
        "message.part.updated" => parse_part_updated(&parsed, logs),
        "session.idle" => parse_session_idle(&parsed, logs),
        "session.status" => {
            parse_session_status(&parsed, logs);
            None
        }
        "message.updated" => {
            parse_message_updated(&parsed, logs);
            None
        }
        "question.asked" => parse_question_asked(&parsed, logs),
        "permission.asked" => parse_permission_asked(&parsed, logs),
        _ => {
            logs.log(format!("  (unhandled event type: {})", event_type));
            None
        }
    }
}

fn parse_part_updated(parsed: &serde_json::Value, logs: &ServerLogs) -> Option<StreamEvent> {
    let props = parsed.get("properties")?;
    let part_data = props.get("part")?;

    let session_id = part_data
        .get("sessionID")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    let part_type = part_data.get("type").and_then(|v| v.as_str()).unwrap_or("");

    logs.log(format!(
        "  part.type={} session={}",
        part_type,
        truncate_str(&session_id, 12)
    ));

    match part_type {
        "text" => {
            let part: Part = serde_json::from_value(part_data.clone()).ok()?;
            if part.text.is_some() {
                Some(StreamEvent::PartUpdated { session_id, part })
            } else {
                None
            }
        }
        "tool" => {
            let tool_name = part_data
                .get("tool")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let status = part_data
                .get("state")
                .and_then(|s| s.get("status"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();
            let input = part_data
                .get("state")
                .and_then(|s| s.get("input"))
                .cloned()
                .unwrap_or(serde_json::Value::Null);

            logs.log(format!(
                "  tool={} status={} input_len={}",
                tool_name,
                status,
                input.to_string().len()
            ));

            Some(StreamEvent::ToolCall {
                session_id,
                tool_name,
                status,
                input,
            })
        }
        "step-start" => {
            logs.log("  (step started)");
            None
        }
        "step-finish" => {
            let reason = part_data
                .get("reason")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            logs.log(format!("  (step finished: {})", reason));
            None
        }
        _ => {
            logs.log(format!("  (unhandled part type: {})", part_type));
            None
        }
    }
}

fn parse_session_idle(parsed: &serde_json::Value, logs: &ServerLogs) -> Option<StreamEvent> {
    let props = parsed.get("properties")?;
    let session_id = props
        .get("sessionID")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    logs.log(format!("  session={}", truncate_str(&session_id, 12)));
    Some(StreamEvent::SessionIdle { session_id })
}

fn parse_session_status(parsed: &serde_json::Value, logs: &ServerLogs) {
    if let Some(props) = parsed.get("properties") {
        let session_id = props
            .get("sessionID")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let status = props
            .get("status")
            .and_then(|s| s.get("type"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");
        logs.log(format!(
            "  session={} status={}",
            truncate_str(session_id, 12),
            status
        ));
    }
}

fn parse_message_updated(parsed: &serde_json::Value, logs: &ServerLogs) {
    if let Some(props) = parsed.get("properties") {
        if let Some(info) = props.get("info") {
            let role = info.get("role").and_then(|v| v.as_str()).unwrap_or("");
            let session_id = info.get("sessionID").and_then(|v| v.as_str()).unwrap_or("");
            logs.log(format!(
                "  role={} session={}",
                role,
                truncate_str(session_id, 12)
            ));
        }
    }
}

fn parse_question_asked(parsed: &serde_json::Value, logs: &ServerLogs) -> Option<StreamEvent> {
    let props = parsed.get("properties")?;

    let request_id = props
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let session_id = props
        .get("sessionID")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let questions: Vec<QuestionInfo> = props
        .get("questions")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    logs.log(format!(
        "  request_id={} session={} questions={}",
        truncate_str(&request_id, 12),
        truncate_str(&session_id, 12),
        questions.len()
    ));

    Some(StreamEvent::QuestionAsked {
        session_id,
        request_id,
        questions,
    })
}

fn parse_permission_asked(parsed: &serde_json::Value, logs: &ServerLogs) -> Option<StreamEvent> {
    let props = parsed.get("properties")?;

    let request_id = props
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let session_id = props
        .get("sessionID")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let permission = props
        .get("permission")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let patterns: Vec<String> = props
        .get("patterns")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();

    logs.log(format!(
        "  request_id={} session={} permission={} patterns={:?}",
        truncate_str(&request_id, 12),
        truncate_str(&session_id, 12),
        permission,
        patterns
    ));

    Some(StreamEvent::PermissionAsked {
        session_id,
        request_id,
        permission,
        patterns,
    })
}

/// Manages a spawned OpenCode server process.
pub struct ServerProcess {
    child: Child,
}

impl ServerProcess {
    /// Starts a new OpenCode server on the given port.
    ///
    /// Waits up to 5 seconds for the server to become healthy.
    pub async fn start(port: u16) -> Result<Self> {
        let child = Command::new("opencode")
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let server = OpenCodeServer::new(port);

        // Wait for server to become healthy
        for _ in 0..50 {
            sleep(Duration::from_millis(100)).await;
            if server.is_healthy().await {
                return Ok(Self { child });
            }
        }

        anyhow::bail!("Server failed to start within 5 seconds")
    }

    /// Stops the server process.
    pub async fn stop(&mut self) -> Result<()> {
        self.child.kill().await?;
        Ok(())
    }
}

impl Drop for ServerProcess {
    fn drop(&mut self) {
        let _ = self.child.start_kill();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_creates_with_correct_url() {
        let server = OpenCodeServer::new(4096);
        assert_eq!(server.base_url(), "http://127.0.0.1:4096");
    }

    #[test]
    fn server_logs_on_creation() {
        let server = OpenCodeServer::new(4096);
        let logs = server.logs.get_logs();
        assert!(!logs.is_empty());
        assert!(logs[0].contains("Creating OpenCodeServer"));
    }
}

use anyhow::Result;
use futures::StreamExt;
use reqwest::Client;
use reqwest_eventsource::{Event, EventSource};
use serde::{Deserialize, Serialize};
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

const DEFAULT_HOST: &str = "127.0.0.1";

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        format!("{}...", chars[..max_chars].iter().collect::<String>())
    } else {
        s.to_string()
    }
}

#[derive(Clone, Default, Debug)]
pub struct ServerLogs {
    logs: Arc<Mutex<Vec<String>>>,
}

impl ServerLogs {
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn log(&self, message: String) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(format!("[{}] {}", timestamp, message));
        }
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.logs.lock().map(|l| l.clone()).unwrap_or_default()
    }
}

#[derive(Debug, Clone)]
pub struct OpenCodeServer {
    client: Client,
    base_url: String,
    pub logs: ServerLogs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(rename = "parentID")]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    #[serde(rename = "sessionID")]
    pub session_id: String,
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Part {
    pub id: String,
    #[serde(rename = "type")]
    pub part_type: String,
    #[serde(default)]
    pub text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageResponse {
    pub info: Message,
    pub parts: Vec<Part>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub healthy: bool,
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub worktree: String,
    #[serde(default)]
    pub vcs: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathResponse {
    pub path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Model {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub models: std::collections::HashMap<String, Model>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderResponse {
    pub all: Vec<Provider>,
    #[serde(default)]
    pub default: serde_json::Value,
    #[serde(default)]
    pub connected: Vec<String>,
}



#[derive(Debug, Serialize)]
struct CreateSessionRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,
}

#[derive(Debug, Serialize)]
struct SendMessageRequest {
    parts: Vec<MessagePart>,
}

#[derive(Debug, Serialize)]
struct MessagePart {
    #[serde(rename = "type")]
    part_type: String,
    text: String,
}

#[derive(Debug, Clone)]
pub enum StreamEvent {
    Connected,
    PartUpdated { session_id: String, part: Part },
    ToolCall { session_id: String, tool_name: String, status: String, input: serde_json::Value },
    SessionIdle { session_id: String },
    QuestionAsked { session_id: String, request_id: String, questions: Vec<QuestionInfo> },
    PermissionAsked { session_id: String, request_id: String, permission: String, patterns: Vec<String> },
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionInfo {
    pub question: String,
    #[serde(default)]
    pub header: Option<String>,
    #[serde(default)]
    pub options: Vec<QuestionOption>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestionOption {
    pub label: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl OpenCodeServer {
    pub fn new(port: u16) -> Self {
        let logs = ServerLogs::new();
        logs.log(format!("Creating OpenCodeServer for port {}", port));
        Self {
            client: Client::new(),
            base_url: format!("http://{}:{}", DEFAULT_HOST, port),
            logs,
        }
    }

    pub async fn health(&self) -> Result<HealthResponse> {
        self.logs.log("GET /global/health".to_string());
        let resp: HealthResponse = self.client
            .get(format!("{}/global/health", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        self.logs.log(format!("Health response: healthy={}, version={}", resp.healthy, resp.version));
        Ok(resp)
    }

    pub async fn is_healthy(&self) -> bool {
        self.health().await.map(|h| h.healthy).unwrap_or(false)
    }

    pub async fn create_session(&self, title: Option<&str>) -> Result<Session> {
        self.logs.log(format!("POST /session (title={:?})", title));
        let req = CreateSessionRequest {
            title: title.map(String::from),
        };
        let response = self.client
            .post(format!("{}/session", self.base_url))
            .json(&req)
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                let status = resp.status();
                self.logs.log(format!("Response status: {}", status));
                
                let body = resp.text().await?;
                self.logs.log(format!("Response body: {}", truncate_str(&body, 200)));
                
                let session: Session = serde_json::from_str(&body)
                    .map_err(|e| anyhow::anyhow!("Failed to parse session: {}. Body: {}", e, truncate_str(&body, 500)))?;
                self.logs.log(format!("Session created: id={}", session.id));
                Ok(session)
            }
            Err(e) => {
                self.logs.log(format!("Request failed: {}", e));
                Err(e.into())
            }
        }
    }

    pub async fn send_message(&self, session_id: &str, text: &str) -> Result<MessageResponse> {
        self.logs.log(format!("POST /session/{}/message", session_id));
        self.logs.log(format!("Message text: {}", truncate_str(text, 100)));
        
        let req = SendMessageRequest {
            parts: vec![MessagePart {
                part_type: "text".to_string(),
                text: text.to_string(),
            }],
        };
        
        let response = self.client
            .post(format!("{}/session/{}/message", self.base_url, session_id))
            .json(&req)
            .send()
            .await;
        
        match response {
            Ok(resp) => {
                let status = resp.status();
                self.logs.log(format!("Response status: {}", status));
                
                let body = resp.text().await?;
                self.logs.log(format!("Response length: {} bytes", body.len()));
                self.logs.log(format!("Response preview: {}", truncate_str(&body, 500)));
                
                let msg_response: MessageResponse = serde_json::from_str(&body)
                    .map_err(|e| anyhow::anyhow!("Failed to parse message response: {}. Body: {}", e, truncate_str(&body, 1000)))?;
                
                self.logs.log(format!("Message response: {} parts", msg_response.parts.len()));
                for (i, part) in msg_response.parts.iter().enumerate() {
                    let text_preview = part.text.as_ref()
                        .map(|t| truncate_str(t, 100))
                        .unwrap_or_else(|| "(no text)".to_string());
                    self.logs.log(format!("  Part {}: type={}, text={}", i, part.part_type, text_preview));
                }
                
                Ok(msg_response)
            }
            Err(e) => {
                self.logs.log(format!("Request failed: {}", e));
                Err(e.into())
            }
        }
    }

    pub async fn send_message_async(&self, session_id: &str, text: &str) -> Result<()> {
        self.logs.log(format!("POST /session/{}/prompt_async", session_id));
        self.logs.log(format!("Message text: {}", truncate_str(text, 100)));
        
        let req = SendMessageRequest {
            parts: vec![MessagePart {
                part_type: "text".to_string(),
                text: text.to_string(),
            }],
        };
        
        let response = self.client
            .post(format!("{}/session/{}/prompt_async", self.base_url, session_id))
            .json(&req)
            .send()
            .await?;
        
        let status = response.status();
        self.logs.log(format!("Response status: {}", status));
        
        if status.is_success() {
            Ok(())
        } else {
            let body = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Async message failed: {} - {}", status, body))
        }
    }

    pub async fn list_projects(&self) -> Result<Vec<Project>> {
        self.logs.log("GET /project".to_string());
        let resp = self.client
            .get(format!("{}/project", self.base_url))
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        let body = resp.text().await?;
        self.logs.log(format!("Response: {}", truncate_str(&body, 200)));
        
        let projects: Vec<Project> = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse projects: {}. Body: {}", e, truncate_str(&body, 500)))?;
        Ok(projects)
    }

    pub async fn get_current_project(&self) -> Result<Project> {
        self.logs.log("GET /project/current".to_string());
        let resp = self.client
            .get(format!("{}/project/current", self.base_url))
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        let body = resp.text().await?;
        self.logs.log(format!("Response: {}", truncate_str(&body, 200)));
        
        let project: Project = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse project: {}. Body: {}", e, truncate_str(&body, 500)))?;
        Ok(project)
    }

    pub async fn get_path(&self) -> Result<String> {
        self.logs.log("GET /path".to_string());
        let resp = self.client
            .get(format!("{}/path", self.base_url))
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        let body = resp.text().await?;
        self.logs.log(format!("Response: {}", truncate_str(&body, 200)));
        
        if let Ok(path_resp) = serde_json::from_str::<PathResponse>(&body) {
            return Ok(path_resp.path);
        }
        if let Ok(path) = serde_json::from_str::<String>(&body) {
            return Ok(path);
        }
        Ok(body.trim_matches('"').to_string())
    }

    pub async fn get_providers(&self) -> Result<ProviderResponse> {
        self.logs.log("GET /provider".to_string());
        let resp = self.client
            .get(format!("{}/provider", self.base_url))
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        let body = resp.text().await?;
        self.logs.log(format!("Response: {}", truncate_str(&body, 500)));
        
        let providers: ProviderResponse = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse providers: {}. Body: {}", e, truncate_str(&body, 500)))?;
        Ok(providers)
    }

    pub async fn get_config(&self) -> Result<serde_json::Value> {
        self.logs.log("GET /config".to_string());
        let resp = self.client
            .get(format!("{}/config", self.base_url))
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        let body = resp.text().await?;
        self.logs.log(format!("Response: {}", truncate_str(&body, 500)));
        
        let config: serde_json::Value = serde_json::from_str(&body)
            .map_err(|e| anyhow::anyhow!("Failed to parse config: {}. Body: {}", e, truncate_str(&body, 500)))?;
        Ok(config)
    }

    pub async fn set_model(&self, provider_id: &str, model_id: &str) -> Result<()> {
        self.logs.log(format!("PATCH /config provider={} model={}", provider_id, model_id));
        
        let body = serde_json::json!({
            "provider": provider_id,
            "model": model_id
        });
        
        let resp = self.client
            .patch(format!("{}/config", self.base_url))
            .json(&body)
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Failed to set model: {} - {}", status, body))
        }
    }

    pub async fn reply_to_question(&self, request_id: &str, answers: Vec<Vec<String>>) -> Result<()> {
        self.logs.log(format!("POST /question/{}/reply", request_id));
        
        let body = serde_json::json!({
            "answers": answers
        });
        
        let resp = self.client
            .post(format!("{}/question/{}/reply", self.base_url, request_id))
            .json(&body)
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Failed to reply to question: {} - {}", status, body))
        }
    }

    pub async fn reply_to_permission(&self, request_id: &str, reply: &str) -> Result<()> {
        self.logs.log(format!("POST /permission/{}/reply reply={}", request_id, reply));
        
        let body = serde_json::json!({
            "reply": reply
        });
        
        let resp = self.client
            .post(format!("{}/permission/{}/reply", self.base_url, request_id))
            .json(&body)
            .send()
            .await?;
        
        let status = resp.status();
        self.logs.log(format!("Response status: {}", status));
        
        if status.is_success() {
            Ok(())
        } else {
            let body = resp.text().await.unwrap_or_default();
            Err(anyhow::anyhow!("Failed to reply to permission: {} - {}", status, body))
        }
    }

    pub fn subscribe_events(&self, tx: mpsc::Sender<StreamEvent>) {
        let url = format!("{}/event", self.base_url);
        let logs = self.logs.clone();
        
        logs.log(format!("Subscribing to SSE: {}", url));
        
        tokio::spawn(async move {
            let mut es = EventSource::get(&url);
            
            while let Some(event) = es.next().await {
                match event {
                    Ok(Event::Open) => {
                        logs.log("SSE connection opened".to_string());
                        let _ = tx.send(StreamEvent::Connected).await;
                    }
                    Ok(Event::Message(msg)) => {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&msg.data) {
                            let event_type = parsed.get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or(&msg.event);
                            
                            // Log everything verbosely
                            logs.log(format!("[SSE] {}", event_type));
                            
                            let stream_event = match event_type {
                                "message.part.updated" => {
                                    if let Some(props) = parsed.get("properties") {
                                        if let Some(part_data) = props.get("part") {
                                            let session_id = part_data.get("sessionID")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("")
                                                .to_string();
                                            
                                            let part_type = part_data.get("type")
                                                .and_then(|v| v.as_str())
                                                .unwrap_or("");
                                            
                                            logs.log(format!("  part.type={} session={}", part_type, truncate_str(&session_id, 12)));
                                            
                                            match part_type {
                                                "text" => {
                                                    let text_preview = part_data.get("text")
                                                        .and_then(|v| v.as_str())
                                                        .map(|s| truncate_str(s, 50))
                                                        .unwrap_or_default();
                                                    logs.log(format!("  text: {}", text_preview));
                                                    
                                                    if let Ok(part) = serde_json::from_value::<Part>(part_data.clone()) {
                                                        if part.text.is_some() {
                                                            Some(StreamEvent::PartUpdated { session_id, part })
                                                        } else {
                                                            None
                                                        }
                                                    } else {
                                                        None
                                                    }
                                                }
                                                "tool" => {
                                                    let tool_name = part_data.get("tool")
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("unknown")
                                                        .to_string();
                                                    let status = part_data.get("state")
                                                        .and_then(|s| s.get("status"))
                                                        .and_then(|v| v.as_str())
                                                        .unwrap_or("unknown")
                                                        .to_string();
                                                    let input = part_data.get("state")
                                                        .and_then(|s| s.get("input"))
                                                        .cloned()
                                                        .unwrap_or(serde_json::Value::Null);
                                                    logs.log(format!("  tool={} status={} input_len={}", tool_name, status, input.to_string().len()));
                                                    
                                                    Some(StreamEvent::ToolCall { 
                                                        session_id,
                                                        tool_name, 
                                                        status,
                                                        input,
                                                    })
                                                }
                                                "step-start" => {
                                                    logs.log("  (step started)".to_string());
                                                    None
                                                }
                                                "step-finish" => {
                                                    let reason = part_data.get("reason")
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
                                        } else {
                                            logs.log("  (no part data)".to_string());
                                            None
                                        }
                                    } else {
                                        logs.log("  (no properties)".to_string());
                                        None
                                    }
                                }
                                "session.idle" => {
                                    if let Some(props) = parsed.get("properties") {
                                        let session_id = props.get("sessionID")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        logs.log(format!("  session={}", truncate_str(&session_id, 12)));
                                        Some(StreamEvent::SessionIdle { session_id })
                                    } else {
                                        None
                                    }
                                }
                                "session.status" => {
                                    if let Some(props) = parsed.get("properties") {
                                        let session_id = props.get("sessionID")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("");
                                        let status = props.get("status")
                                            .and_then(|s| s.get("type"))
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("unknown");
                                        logs.log(format!("  session={} status={}", truncate_str(session_id, 12), status));
                                    }
                                    None
                                }
                                "message.updated" => {
                                    if let Some(props) = parsed.get("properties") {
                                        if let Some(info) = props.get("info") {
                                            let role = info.get("role").and_then(|v| v.as_str()).unwrap_or("");
                                            let session_id = info.get("sessionID").and_then(|v| v.as_str()).unwrap_or("");
                                            logs.log(format!("  role={} session={}", role, truncate_str(session_id, 12)));
                                        }
                                    }
                                    None
                                }
                                "question.asked" => {
                                    if let Some(props) = parsed.get("properties") {
                                        let request_id = props.get("id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let session_id = props.get("sessionID")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let questions: Vec<QuestionInfo> = props.get("questions")
                                            .and_then(|v| serde_json::from_value(v.clone()).ok())
                                            .unwrap_or_default();
                                        
                                        logs.log(format!("  request_id={} session={} questions={}", 
                                            truncate_str(&request_id, 12), 
                                            truncate_str(&session_id, 12),
                                            questions.len()
                                        ));
                                        
                                        Some(StreamEvent::QuestionAsked { session_id, request_id, questions })
                                    } else {
                                        None
                                    }
                                }
                                "permission.asked" => {
                                    if let Some(props) = parsed.get("properties") {
                                        let request_id = props.get("id")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let session_id = props.get("sessionID")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let permission = props.get("permission")
                                            .and_then(|v| v.as_str())
                                            .unwrap_or("")
                                            .to_string();
                                        let patterns: Vec<String> = props.get("patterns")
                                            .and_then(|v| v.as_array())
                                            .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                                            .unwrap_or_default();
                                        
                                        logs.log(format!("  request_id={} session={} permission={} patterns={:?}", 
                                            truncate_str(&request_id, 12), 
                                            truncate_str(&session_id, 12),
                                            permission,
                                            patterns
                                        ));
                                        
                                        Some(StreamEvent::PermissionAsked { session_id, request_id, permission, patterns })
                                    } else {
                                        None
                                    }
                                }
                                _ => {
                                    logs.log(format!("  (event type: {})", event_type));
                                    None
                                }
                            };
                            
                            if let Some(evt) = stream_event {
                                if tx.send(evt).await.is_err() {
                                    logs.log("SSE receiver dropped, closing connection".to_string());
                                    break;
                                }
                            }
                        } else {
                            logs.log(format!("[SSE RAW] {}: {}", msg.event, truncate_str(&msg.data, 100)));
                        }
                    }
                    Err(e) => {
                        logs.log(format!("SSE error: {}", e));
                        let _ = tx.send(StreamEvent::Error(e.to_string())).await;
                    }
                }
            }
            
            logs.log("SSE connection closed".to_string());
        });
    }

}

pub struct ServerProcess {
    child: Child,
}

impl ServerProcess {
    pub async fn start(port: u16) -> Result<Self> {
        let child = Command::new("opencode")
            .arg("serve")
            .arg("--port")
            .arg(port.to_string())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;

        let server = OpenCodeServer::new(port);
        
        for _ in 0..50 {
            sleep(Duration::from_millis(100)).await;
            if server.is_healthy().await {
                return Ok(Self { child });
            }
        }

        anyhow::bail!("Server failed to start within 5 seconds")
    }

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

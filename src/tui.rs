use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, MouseEventKind, EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use tokio::sync::mpsc;

use crate::orchestrator::{Orchestrator, TaskPlan};
use crate::server::{OpenCodeServer, ServerProcess};

fn truncate_str(s: &str, max_chars: usize) -> String {
    let chars: Vec<char> = s.chars().collect();
    if chars.len() > max_chars {
        format!("{}...", chars[..max_chars].iter().collect::<String>())
    } else {
        s.to_string()
    }
}

fn extract_question_text(input: &serde_json::Value) -> String {
    // Handle case where input is a string that needs to be parsed
    if let Some(input_str) = input.as_str() {
        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(input_str) {
            return extract_question_text(&parsed);
        }
    }
    
    if let Some(q) = input.get("question").and_then(|v| v.as_str()) {
        return q.to_string();
    }
    
    if let Some(questions) = input.get("questions").and_then(|v| v.as_array()) {
        let texts: Vec<&str> = questions
            .iter()
            .filter_map(|q| q.get("question").and_then(|v| v.as_str()))
            .collect();
        return texts.join("\n");
    }
    
    String::new()
}

#[derive(Clone, PartialEq, Debug)]
enum WorkerState {
    Starting,
    Running,
    WaitingForInput,
    Complete,
    Error,
}

struct Worker {
    id: u32,
    description: String,
    session_id: Option<String>,
    state: WorkerState,
    output: Vec<String>,
    streaming_content: String,
    current_tool: Option<String>,
    tool_history: Vec<String>,
    pending_question: Option<String>,
    pending_question_request_id: Option<String>,
}

impl Worker {
    fn new(id: u32, description: String) -> Self {
        Self {
            id,
            description,
            session_id: None,
            state: WorkerState::Starting,
            output: Vec::new(),
            streaming_content: String::new(),
            current_tool: None,
            tool_history: Vec::new(),
            pending_question: None,
            pending_question_request_id: None,
        }
    }

    fn get_display_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        
        for tool in &self.tool_history {
            lines.push(format!("✓ {}", tool));
        }
        
        if let Some(tool) = &self.current_tool {
            lines.push(format!("⚙ {}...", tool));
        }
        
        if !self.tool_history.is_empty() || self.current_tool.is_some() {
            lines.push(String::new());
        }
        
        if !self.streaming_content.is_empty() {
            for line in self.streaming_content.lines() {
                lines.push(line.to_string());
            }
        } else {
            for line in &self.output {
                lines.push(line.clone());
            }
        }
        
        lines
    }

    fn get_summary(&self) -> String {
        let content = if !self.streaming_content.is_empty() {
            &self.streaming_content
        } else {
            &self.output.join("\n")
        };
        
        let lines: Vec<&str> = content.lines()
            .filter(|l| !l.trim().is_empty())
            .filter(|l| !l.starts_with("✓") && !l.starts_with("⚙") && !l.starts_with("⏳"))
            .collect();
        
        if lines.is_empty() {
            return format!("Worker #{} completed (no output)", self.id);
        }
        
        let summary: String = lines.iter()
            .take(10)
            .map(|s| *s)
            .collect::<Vec<&str>>()
            .join("\n");
        
        if lines.len() > 10 {
            format!("{}...", summary)
        } else {
            summary
        }
    }
}

struct Session {
    id: usize,
    name: String,
    messages: Vec<(String, bool)>,
    workers: Vec<Worker>,
    selected_worker: Option<usize>,
    scroll_offset: usize,
    orchestrator_session_id: Option<String>,
}

#[derive(Clone)]
struct PendingPermission {
    request_id: String,
    #[allow(dead_code)]
    session_id: String,
    permission: String,
    patterns: Vec<String>,
    worker_id: Option<u32>,
    worker_description: Option<String>,
}

impl Session {
    fn new(id: usize, name: String) -> Self {
        let welcome_msg = format!("Session '{}' created.", &name);
        Self {
            id,
            name,
            messages: vec![
                (welcome_msg, false),
                ("Type a task and press Enter to start.".to_string(), false),
                ("".to_string(), false),
            ],
            workers: Vec::new(),
            selected_worker: None,
            scroll_offset: 0,
            orchestrator_session_id: None,
        }
    }

    fn select_next_worker(&mut self) {
        if self.workers.is_empty() {
            return;
        }
        self.selected_worker = Some(match self.selected_worker {
            Some(i) if i < self.workers.len() - 1 => i + 1,
            _ => 0,
        });
    }

    fn select_prev_worker(&mut self) {
        if self.workers.is_empty() {
            return;
        }
        self.selected_worker = Some(match self.selected_worker {
            Some(i) if i > 0 => i - 1,
            _ => self.workers.len() - 1,
        });
    }
}

enum AppMessage {
    OrchestratorLog(usize, String),
    ServerLogs(Vec<String>),
    TaskPlan(usize, TaskPlan, Vec<String>, String),
    WorkerStarted(usize, u32, String),
    WorkerOutput(usize, u32, String),
    #[allow(dead_code)]
    WorkerComplete(usize, u32),
    WorkerError(usize, u32, String),
    StreamEvent(crate::server::StreamEvent),
    CommandResult(String),
    Error(String),
    ModelsLoaded(Vec<ModelOption>),
    ReportToOrchestrator(usize, String),
}

enum SlashCommand {
    Help,
    Projects,
    ProjectCurrent,
    Path,
    Clear,
    NewSession(Option<String>),
    ListSessions,
    RenameSession(String),
    DeleteSession,
    Models,
    ModelSelect,
    ModelSet(String, String),
    Reply(u32, String),
    Unknown(String),
}

#[derive(Clone)]
struct ModelOption {
    provider_id: String,
    provider_name: String,
    model_id: String,
    model_name: String,
}

struct CommandSuggestion {
    command: &'static str,
    description: &'static str,
}

const COMMAND_SUGGESTIONS: &[CommandSuggestion] = &[
    CommandSuggestion { command: "/help", description: "Show available commands" },
    CommandSuggestion { command: "/new", description: "Create new session" },
    CommandSuggestion { command: "/sessions", description: "List all sessions" },
    CommandSuggestion { command: "/rename", description: "Rename current session" },
    CommandSuggestion { command: "/delete", description: "Delete current session" },
    CommandSuggestion { command: "/models", description: "List available models" },
    CommandSuggestion { command: "/model", description: "Set model (provider/model)" },
    CommandSuggestion { command: "/reply", description: "Reply to worker (/reply #N message)" },
    CommandSuggestion { command: "/projects", description: "List all projects" },
    CommandSuggestion { command: "/project current", description: "Show current project" },
    CommandSuggestion { command: "/path", description: "Show current working path" },
    CommandSuggestion { command: "/clear", description: "Clear chat messages" },
];

fn get_suggestions(input: &str) -> Vec<&'static CommandSuggestion> {
    if !input.starts_with('/') {
        return vec![];
    }
    
    let search = input.to_lowercase();
    COMMAND_SUGGESTIONS
        .iter()
        .filter(|s| s.command.starts_with(&search))
        .collect()
}

fn format_tool_display(tool_name: &str, input: &serde_json::Value) -> String {
    if input.is_null() || input.as_object().map(|o| o.is_empty()).unwrap_or(true) {
        tool_name.to_string()
    } else {
        format!("{} {}", tool_name, input)
    }
}

fn parse_slash_command(input: &str) -> Option<SlashCommand> {
    let trimmed = input.trim();
    if !trimmed.starts_with('/') {
        return None;
    }
    
    let parts: Vec<&str> = trimmed[1..].split_whitespace().collect();
    if parts.is_empty() {
        return Some(SlashCommand::Help);
    }
    
    match parts[0].to_lowercase().as_str() {
        "help" | "h" | "?" => Some(SlashCommand::Help),
        "new" | "n" => {
            let name = if parts.len() > 1 {
                Some(parts[1..].join(" "))
            } else {
                None
            };
            Some(SlashCommand::NewSession(name))
        }
        "sessions" | "ls" => Some(SlashCommand::ListSessions),
        "rename" | "mv" => {
            if parts.len() > 1 {
                Some(SlashCommand::RenameSession(parts[1..].join(" ")))
            } else {
                Some(SlashCommand::Unknown("rename requires a name".to_string()))
            }
        }
        "delete" | "del" | "rm" => Some(SlashCommand::DeleteSession),
        "models" => Some(SlashCommand::Models),
        "model" | "m" => {
            if parts.len() > 1 {
                let model_spec = parts[1..].join(" ");
                if let Some((provider, model)) = model_spec.split_once('/') {
                    Some(SlashCommand::ModelSet(provider.to_string(), model.to_string()))
                } else {
                    Some(SlashCommand::Unknown("model requires provider/model format".to_string()))
                }
            } else {
                Some(SlashCommand::ModelSelect)
            }
        }
        "projects" | "project" | "proj" | "p" => {
            if parts.len() > 1 && parts[1] == "current" {
                Some(SlashCommand::ProjectCurrent)
            } else {
                Some(SlashCommand::Projects)
            }
        }
        "path" | "pwd" => Some(SlashCommand::Path),
        "clear" | "cls" => Some(SlashCommand::Clear),
        "reply" | "r" => {
            if parts.len() > 2 {
                let worker_str = parts[1].trim_start_matches('#');
                if let Ok(worker_id) = worker_str.parse::<u32>() {
                    let message = parts[2..].join(" ");
                    Some(SlashCommand::Reply(worker_id, message))
                } else {
                    Some(SlashCommand::Unknown("reply requires worker number (e.g., /reply #1 yes)".to_string()))
                }
            } else {
                Some(SlashCommand::Unknown("reply requires worker number and message (e.g., /reply #1 yes)".to_string()))
            }
        }
        cmd => Some(SlashCommand::Unknown(cmd.to_string())),
    }
}

struct App {
    #[allow(dead_code)]
    server: OpenCodeServer,
    #[allow(dead_code)]
    orchestrator: Orchestrator,
    sessions: Vec<Session>,
    current_session: usize,
    next_session_id: usize,
    input: String,
    cursor_pos: usize,
    input_mode: bool,
    orchestrator_logs: Vec<String>,
    logs_scroll: usize,
    quit: bool,
    status: String,
    show_logs: bool,
    confirm_delete: bool,
    confirm_clear_all: bool,
    confirm_delete_session: bool,
    autocomplete_index: usize,
    show_autocomplete: bool,
    show_model_selector: bool,
    model_options: Vec<ModelOption>,
    model_selector_index: usize,
    pending_permissions: Vec<PendingPermission>,
    show_permission_dialog: bool,
    permission_selector_index: usize,
}

impl App {
    fn new(server: OpenCodeServer) -> Self {
        let orchestrator = Orchestrator::new(server.clone());
        let initial_session = Session::new(0, "Session 1".to_string());
        Self {
            server,
            orchestrator,
            sessions: vec![initial_session],
            current_session: 0,
            next_session_id: 1,
            input: String::new(),
            cursor_pos: 0,
            input_mode: true,
            orchestrator_logs: Vec::new(),
            logs_scroll: 0,
            quit: false,
            status: "Ready".to_string(),
            show_logs: false,
            confirm_delete: false,
            confirm_clear_all: false,
            confirm_delete_session: false,
            autocomplete_index: 0,
            show_autocomplete: false,
            show_model_selector: false,
            model_options: Vec::new(),
            model_selector_index: 0,
            pending_permissions: Vec::new(),
            show_permission_dialog: false,
            permission_selector_index: 0,
        }
    }

    fn current_session(&self) -> &Session {
        &self.sessions[self.current_session]
    }

    fn current_session_mut(&mut self) -> &mut Session {
        &mut self.sessions[self.current_session]
    }

    fn create_session(&mut self, name: Option<String>) {
        let name = name.unwrap_or_else(|| format!("Session {}", self.next_session_id + 1));
        let session = Session::new(self.next_session_id, name.clone());
        self.next_session_id += 1;
        self.sessions.push(session);
        self.current_session = self.sessions.len() - 1;
        self.status = format!("Created session '{}'", name);
    }

    fn delete_current_session(&mut self) {
        if self.sessions.len() <= 1 {
            self.status = "Cannot delete the only session".to_string();
            return;
        }
        let name = self.sessions[self.current_session].name.clone();
        self.sessions.remove(self.current_session);
        if self.current_session >= self.sessions.len() {
            self.current_session = self.sessions.len() - 1;
        }
        self.status = format!("Deleted session '{}'", name);
    }

    fn next_session(&mut self) {
        if !self.sessions.is_empty() {
            self.current_session = (self.current_session + 1) % self.sessions.len();
        }
    }

    fn prev_session(&mut self) {
        if !self.sessions.is_empty() {
            self.current_session = if self.current_session == 0 {
                self.sessions.len() - 1
            } else {
                self.current_session - 1
            };
        }
    }

    fn get_current_suggestions(&self) -> Vec<&'static CommandSuggestion> {
        if self.input.starts_with('/') {
            get_suggestions(&self.input)
        } else {
            vec![]
        }
    }

    fn apply_autocomplete(&mut self) {
        let suggestions = self.get_current_suggestions();
        if !suggestions.is_empty() && self.autocomplete_index < suggestions.len() {
            self.input = suggestions[self.autocomplete_index].command.to_string();
            self.cursor_pos = self.input.chars().count();
            self.show_autocomplete = false;
        }
    }

    fn autocomplete_next(&mut self) {
        let suggestions = self.get_current_suggestions();
        if !suggestions.is_empty() {
            self.autocomplete_index = (self.autocomplete_index + 1) % suggestions.len();
        }
    }

    fn autocomplete_prev(&mut self) {
        let suggestions = self.get_current_suggestions();
        if !suggestions.is_empty() {
            self.autocomplete_index = if self.autocomplete_index == 0 {
                suggestions.len() - 1
            } else {
                self.autocomplete_index - 1
            };
        }
    }

    fn find_session_by_worker_session_id(&mut self, opencode_session_id: &str) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| {
            s.workers.iter().any(|w| w.session_id.as_ref() == Some(&opencode_session_id.to_string()))
        })
    }
}

pub async fn run_tui(_num_agents: usize, _workdir: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let area = f.area();
        let block = Paragraph::new("Starting opencode server...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(block, area);
    })?;

    let port = 14096u16;
    let mut server_process: ServerProcess = ServerProcess::start(port).await?;
    let server = OpenCodeServer::new(port);

    let mut app = App::new(server.clone());
    
    app.status = "Initializing orchestrator...".to_string();
    terminal.draw(|f| ui(f, &app))?;
    
    app.orchestrator.init().await?;
    app.status = "Ready - Type your task".to_string();

    let (tx, mut rx) = mpsc::channel::<AppMessage>(100);

    // Subscribe to SSE events for real-time streaming
    let (sse_tx, mut sse_rx) = mpsc::channel::<crate::server::StreamEvent>(100);
    server.subscribe_events(sse_tx);
    
    // Forward SSE events to main channel
    let tx_sse = tx.clone();
    tokio::spawn(async move {
        while let Some(event) = sse_rx.recv().await {
            if tx_sse.send(AppMessage::StreamEvent(event)).await.is_err() {
                break;
            }
        }
    });

    loop {
        terminal.draw(|f| ui(f, &app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Mouse(mouse) => {
                    if app.show_logs {
                        match mouse.kind {
                            MouseEventKind::ScrollDown => {
                                app.logs_scroll = app.logs_scroll.saturating_add(3);
                            }
                            MouseEventKind::ScrollUp => {
                                app.logs_scroll = app.logs_scroll.saturating_sub(3);
                            }
                            _ => {}
                        }
                    }
                }
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    if app.input_mode {
                        match key.code {
                            KeyCode::Esc => {
                                if app.show_autocomplete {
                                    app.show_autocomplete = false;
                                } else {
                                    app.input_mode = false;
                                }
                            }
                            KeyCode::Enter => {
                                if app.show_autocomplete && !app.get_current_suggestions().is_empty() {
                                    app.apply_autocomplete();
                                } else if !app.input.is_empty() {
                                    let message = app.input.clone();
                                    app.input.clear();
                                    app.cursor_pos = 0;
                                    app.show_autocomplete = false;
                                    app.autocomplete_index = 0;
                                    
                                    if let Some(cmd) = parse_slash_command(&message) {
                                        app.current_session_mut().messages.push((format!("→ {}", message), true));
                                        match cmd {
                                            SlashCommand::Help => {
                                                let session = app.current_session_mut();
                                                session.messages.push(("".to_string(), false));
                                                session.messages.push(("Session commands:".to_string(), false));
                                                session.messages.push(("  /new [name]       - Create new session".to_string(), false));
                                                session.messages.push(("  /sessions, /ls    - List all sessions".to_string(), false));
                                                session.messages.push(("  /rename <name>    - Rename current session".to_string(), false));
                                                session.messages.push(("  /delete, /rm      - Delete current session".to_string(), false));
                                                session.messages.push(("".to_string(), false));
                                                session.messages.push(("Model commands:".to_string(), false));
                                                session.messages.push(("  /models           - List available models".to_string(), false));
                                                session.messages.push(("  /model <p>/<m>    - Set model (e.g. /model anthropic/claude-3)".to_string(), false));
                                                session.messages.push(("".to_string(), false));
                                                session.messages.push(("Other commands:".to_string(), false));
                                                session.messages.push(("  /help, /h, /?     - Show this help".to_string(), false));
                                                session.messages.push(("  /projects, /p     - List all projects".to_string(), false));
                                                session.messages.push(("  /project current  - Show current project".to_string(), false));
                                                session.messages.push(("  /path, /pwd       - Show current path".to_string(), false));
                                                session.messages.push(("  /clear, /cls      - Clear chat messages".to_string(), false));
                                                session.messages.push(("".to_string(), false));
                                                session.messages.push(("Navigation: n/p to switch sessions".to_string(), false));
                                                session.messages.push(("".to_string(), false));
                                            }
                                            SlashCommand::NewSession(name) => {
                                                app.create_session(name);
                                            }
                                            SlashCommand::ListSessions => {
                                                let session_list: Vec<String> = app.sessions.iter().enumerate().map(|(i, s)| {
                                                    let marker = if i == app.current_session { "→" } else { " " };
                                                    let workers = s.workers.len();
                                                    format!("{} {}: {} ({} workers)", marker, i + 1, s.name, workers)
                                                }).collect();
                                                let session = app.current_session_mut();
                                                session.messages.push(("Sessions:".to_string(), false));
                                                for line in session_list {
                                                    session.messages.push((format!("  {}", line), false));
                                                }
                                            }
                                            SlashCommand::RenameSession(name) => {
                                                app.current_session_mut().name = name.clone();
                                                app.status = format!("Session renamed to '{}'", name);
                                            }
                                            SlashCommand::DeleteSession => {
                                                if app.sessions.len() <= 1 {
                                                    app.current_session_mut().messages.push(("Cannot delete the only session.".to_string(), false));
                                                } else {
                                                    app.confirm_delete_session = true;
                                                    app.status = format!("Delete session '{}'? (y/n)", app.current_session().name);
                                                }
                                            }
                                            SlashCommand::Clear => {
                                                let session = app.current_session_mut();
                                                session.messages.clear();
                                                session.messages.push(("Chat cleared.".to_string(), false));
                                            }
                                            SlashCommand::Projects => {
                                                let server_clone = server.clone();
                                                let tx_clone = tx.clone();
                                                app.status = "Fetching projects...".to_string();
                                                tokio::spawn(async move {
                                                    match server_clone.list_projects().await {
                                                        Ok(projects) => {
                                                            if projects.is_empty() {
                                                                let _ = tx_clone.send(AppMessage::CommandResult("No projects found.".to_string())).await;
                                                            } else {
                                                                let _ = tx_clone.send(AppMessage::CommandResult(format!("Projects ({}):", projects.len()))).await;
                                                                for p in projects {
                                                                    let vcs = p.vcs.unwrap_or_else(|| "none".to_string());
                                                                    let _ = tx_clone.send(AppMessage::CommandResult(format!("  • {} ({})", p.worktree, vcs))).await;
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to list projects: {}", e))).await;
                                                        }
                                                    }
                                                });
                                            }
                                            SlashCommand::ProjectCurrent => {
                                                let server_clone = server.clone();
                                                let tx_clone = tx.clone();
                                                app.status = "Fetching current project...".to_string();
                                                tokio::spawn(async move {
                                                    match server_clone.get_current_project().await {
                                                        Ok(project) => {
                                                            let vcs = project.vcs.unwrap_or_else(|| "none".to_string());
                                                            let _ = tx_clone.send(AppMessage::CommandResult(format!("Current project: {} ({})", project.worktree, vcs))).await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to get current project: {}", e))).await;
                                                        }
                                                    }
                                                });
                                            }
                                            SlashCommand::Path => {
                                                let server_clone = server.clone();
                                                let tx_clone = tx.clone();
                                                app.status = "Fetching path...".to_string();
                                                tokio::spawn(async move {
                                                    match server_clone.get_path().await {
                                                        Ok(path) => {
                                                            let _ = tx_clone.send(AppMessage::CommandResult(format!("Current path: {}", path))).await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to get path: {}", e))).await;
                                                        }
                                                    }
                                                });
                                            }
                                            SlashCommand::Models => {
                                                let server_clone = server.clone();
                                                let tx_clone = tx.clone();
                                                app.status = "Fetching models...".to_string();
                                                tokio::spawn(async move {
                                                    match server_clone.get_providers().await {
                                                        Ok(provider_resp) => {
                                                            let _ = tx_clone.send(AppMessage::CommandResult("".to_string())).await;
                                                            let _ = tx_clone.send(AppMessage::CommandResult(format!("Connected providers: {}", provider_resp.connected.join(", ")))).await;
                                                            let _ = tx_clone.send(AppMessage::CommandResult("".to_string())).await;
                                                            
                                                            for provider in &provider_resp.all {
                                                                if provider_resp.connected.contains(&provider.id) {
                                                                    let name = provider.name.as_ref().unwrap_or(&provider.id);
                                                                    let _ = tx_clone.send(AppMessage::CommandResult(format!("{}:", name))).await;
                                                                    for (_model_key, model) in &provider.models {
                                                                        let model_name = model.name.as_ref().unwrap_or(&model.id);
                                                                        let _ = tx_clone.send(AppMessage::CommandResult(format!("  /model {}/{}", provider.id, model.id))).await;
                                                                        if model_name != &model.id {
                                                                            let _ = tx_clone.send(AppMessage::CommandResult(format!("    ({})", model_name))).await;
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                            
                                                            if let Ok(config) = server_clone.get_config().await {
                                                                let current_provider = config.get("provider").and_then(|v| v.as_str()).unwrap_or("?");
                                                                let current_model = config.get("model").and_then(|v| v.as_str()).unwrap_or("?");
                                                                let _ = tx_clone.send(AppMessage::CommandResult("".to_string())).await;
                                                                let _ = tx_clone.send(AppMessage::CommandResult(format!("Current: {}/{}", current_provider, current_model))).await;
                                                            }
                                                        }
                                                        Err(e) => {
                                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to list models: {}", e))).await;
                                                        }
                                                    }
                                                });
                                            }
                                            SlashCommand::ModelSelect => {
                                                let server_clone = server.clone();
                                                let tx_clone = tx.clone();
                                                app.status = "Loading models...".to_string();
                                                tokio::spawn(async move {
                                                    match server_clone.get_providers().await {
                                                        Ok(provider_resp) => {
                                                            let mut options = Vec::new();
                                                            for provider in &provider_resp.all {
                                                                if provider_resp.connected.contains(&provider.id) {
                                                                    let provider_name = provider.name.as_ref().unwrap_or(&provider.id).clone();
                                                                    for (_key, model) in &provider.models {
                                                                        let model_name = model.name.as_ref().unwrap_or(&model.id).clone();
                                                                        options.push(ModelOption {
                                                                            provider_id: provider.id.clone(),
                                                                            provider_name: provider_name.clone(),
                                                                            model_id: model.id.clone(),
                                                                            model_name,
                                                                        });
                                                                    }
                                                                }
                                                            }
                                                            let _ = tx_clone.send(AppMessage::ModelsLoaded(options)).await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to load models: {}", e))).await;
                                                        }
                                                    }
                                                });
                                            }
                                            SlashCommand::ModelSet(provider_id, model_id) => {
                                                let server_clone = server.clone();
                                                let tx_clone = tx.clone();
                                                app.status = format!("Setting model to {}/{}...", provider_id, model_id);
                                                tokio::spawn(async move {
                                                    match server_clone.set_model(&provider_id, &model_id).await {
                                                        Ok(()) => {
                                                            let _ = tx_clone.send(AppMessage::CommandResult(format!("Model set to {}/{}", provider_id, model_id))).await;
                                                        }
                                                        Err(e) => {
                                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to set model: {}", e))).await;
                                                        }
                                                    }
                                                });
                                            }
                                            SlashCommand::Reply(worker_id, reply_message) => {
                                                let session = app.current_session_mut();
                                                if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
                                                    if worker.state == WorkerState::WaitingForInput {
                                                        if let Some(request_id) = worker.pending_question_request_id.clone() {
                                                            worker.state = WorkerState::Running;
                                                            worker.pending_question = None;
                                                            worker.pending_question_request_id = None;
                                                            session.messages.push((format!("→ [To Worker #{}] {}", worker_id, reply_message), true));
                                                            
                                                            let server_clone = server.clone();
                                                            let tx_clone = tx.clone();
                                                            let ui_session_id = session.id;
                                                            app.status = format!("Sending reply to Worker #{}...", worker_id);
                                                            
                                                            tokio::spawn(async move {
                                                                let _ = tx_clone.send(AppMessage::OrchestratorLog(ui_session_id, format!("[REPLY] Replying to question {}", &request_id))).await;
                                                                let answers = vec![vec![reply_message.clone()]];
                                                                match server_clone.reply_to_question(&request_id, answers).await {
                                                                    Ok(()) => {
                                                                        let _ = tx_clone.send(AppMessage::OrchestratorLog(ui_session_id, format!("[REPLY] Sent OK to worker #{}", worker_id))).await;
                                                                    }
                                                                    Err(e) => {
                                                                        let _ = tx_clone.send(AppMessage::Error(format!("Failed to send reply to worker #{}: {}", worker_id, e))).await;
                                                                    }
                                                                }
                                                            });
                                                        } else {
                                                            session.messages.push((format!("✗ Worker #{} has no pending question request", worker_id), false));
                                                        }
                                                    } else {
                                                        session.messages.push((format!("✗ Worker #{} is not waiting for input (state: {:?})", worker_id, worker.state), false));
                                                    }
                                                } else {
                                                    session.messages.push((format!("✗ Worker #{} not found", worker_id), false));
                                                }
                                            }
                                            SlashCommand::Unknown(cmd) => {
                                                app.current_session_mut().messages.push((format!("Unknown command: /{}. Type /help for available commands.", cmd), false));
                                            }
                                        }
                                        continue;
                                    }
                                    
                                    let session_id = app.current_session().id;
                                    let existing_orch_session = app.current_session().orchestrator_session_id.clone();
                                    app.current_session_mut().messages.push((format!("→ {}", message), true));
                                    app.status = "Orchestrator analyzing...".to_string();
                                    
                                    let orchestrator_server = server.clone();
                                    let tx_clone = tx.clone();
                                    let msg = message.clone();
                                    
                                    tokio::spawn(async move {
                                        let _ = tx_clone.send(AppMessage::OrchestratorLog(session_id, "Starting orchestrator...".to_string())).await;
                                        
                                        let mut orch = Orchestrator::new(orchestrator_server.clone());
                                        
                                        if let Some(orch_session_id) = existing_orch_session {
                                            orch.set_session_id(orch_session_id);
                                        } else {
                                            if let Err(e) = orch.init().await {
                                                for log in orch.get_logs() {
                                                    let _ = tx_clone.send(AppMessage::OrchestratorLog(session_id, log.clone())).await;
                                                }
                                                let _ = tx_clone.send(AppMessage::Error(format!("Orchestrator init failed: {}", e))).await;
                                                return;
                                            }
                                        }
                                        
                                        for log in orch.get_logs() {
                                            let _ = tx_clone.send(AppMessage::OrchestratorLog(session_id, log.clone())).await;
                                        }
                                        
                                        let orch_session_id = orch.get_session_id().cloned().unwrap_or_default();
                                        
                                        match orch.plan_tasks(&msg).await {
                                            Ok(plan) => {
                                                let logs = orch.get_logs().to_vec();
                                                let _ = tx_clone.send(AppMessage::TaskPlan(session_id, plan.clone(), logs, orch_session_id)).await;
                                                
                                                for task in plan.tasks {
                                                    let server = orchestrator_server.clone();
                                                    let tx = tx_clone.clone();
                                                    let task_id = task.id;
                                                    let prompt = task.prompt.clone();
                                                    
                                                    let _ = tx_clone.send(AppMessage::OrchestratorLog(session_id, format!("[SPAWN] Spawning worker #{}", task_id))).await;
                                                    
                                                    tokio::spawn(async move {
                                                        let _ = tx.send(AppMessage::OrchestratorLog(session_id, format!("[SPAWN] Inside spawn for worker #{}", task_id))).await;
                                                        let _ = tx.send(AppMessage::WorkerOutput(session_id, task_id, format!("Creating session for worker #{}...", task_id))).await;
                                                        
                                                        match server.create_session(Some(&format!("Worker {}", task_id))).await {
                                                            Ok(session) => {
                                                                let _ = tx.send(AppMessage::OrchestratorLog(session_id, format!("[SPAWN] Worker #{} got session {}", task_id, &session.id[..12]))).await;
                                                                let _ = tx.send(AppMessage::ServerLogs(server.logs.get_logs())).await;
                                                                let _ = tx.send(AppMessage::WorkerStarted(session_id, task_id, session.id.clone())).await;
                                                                let _ = tx.send(AppMessage::WorkerOutput(session_id, task_id, format!("Session: {}", truncate_str(&session.id, 8)))).await;
                                                                let _ = tx.send(AppMessage::WorkerOutput(session_id, task_id, "⏳ Streaming response...".to_string())).await;
                                                                
                                                                match server.send_message_async(&session.id, &prompt).await {
                                                                    Ok(()) => {
                                                                        let _ = tx.send(AppMessage::ServerLogs(server.logs.get_logs())).await;
                                                                    }
                                                                    Err(e) => {
                                                                        let _ = tx.send(AppMessage::ServerLogs(server.logs.get_logs())).await;
                                                                        let _ = tx.send(AppMessage::WorkerError(session_id, task_id, format!("send_message_async failed: {}", e))).await;
                                                                    }
                                                                }
                                                            }
                                                            Err(e) => {
                                                                let _ = tx.send(AppMessage::ServerLogs(server.logs.get_logs())).await;
                                                                let _ = tx.send(AppMessage::WorkerError(session_id, task_id, format!("create_session failed: {}", e))).await;
                                                            }
                                                        }
                                                    });
                                                }
                                            }
                                            Err(e) => {
                                                for log in orch.get_logs() {
                                                    let _ = tx_clone.send(AppMessage::OrchestratorLog(session_id, log.clone())).await;
                                                }
                                                let _ = tx_clone.send(AppMessage::Error(format!("Planning failed: {}", e))).await;
                                            }
                                        }
                                    });
                                }
                            }
                            KeyCode::Tab => {
                                if app.show_autocomplete && !app.get_current_suggestions().is_empty() {
                                    app.apply_autocomplete();
                                } else if app.input.starts_with('/') {
                                    app.show_autocomplete = true;
                                    app.autocomplete_index = 0;
                                }
                            }
                            KeyCode::Down if app.show_autocomplete => {
                                app.autocomplete_next();
                            }
                            KeyCode::Up if app.show_autocomplete => {
                                app.autocomplete_prev();
                            }
                            KeyCode::Left => {
                                if app.cursor_pos > 0 {
                                    app.cursor_pos -= 1;
                                }
                            }
                            KeyCode::Right => {
                                if app.cursor_pos < app.input.chars().count() {
                                    app.cursor_pos += 1;
                                }
                            }
                            KeyCode::Home => {
                                app.cursor_pos = 0;
                            }
                            KeyCode::End => {
                                app.cursor_pos = app.input.chars().count();
                            }
                            KeyCode::Backspace => {
                                if app.cursor_pos > 0 {
                                    let idx = app.input.char_indices()
                                        .nth(app.cursor_pos - 1)
                                        .map(|(i, _)| i)
                                        .unwrap_or(0);
                                    let end_idx = app.input.char_indices()
                                        .nth(app.cursor_pos)
                                        .map(|(i, _)| i)
                                        .unwrap_or(app.input.len());
                                    app.input = format!("{}{}", &app.input[..idx], &app.input[end_idx..]);
                                    app.cursor_pos -= 1;
                                }
                                app.autocomplete_index = 0;
                                app.show_autocomplete = app.input.starts_with('/');
                            }
                            KeyCode::Delete => {
                                let char_count = app.input.chars().count();
                                if app.cursor_pos < char_count {
                                    let idx = app.input.char_indices()
                                        .nth(app.cursor_pos)
                                        .map(|(i, _)| i)
                                        .unwrap_or(app.input.len());
                                    let end_idx = app.input.char_indices()
                                        .nth(app.cursor_pos + 1)
                                        .map(|(i, _)| i)
                                        .unwrap_or(app.input.len());
                                    app.input = format!("{}{}", &app.input[..idx], &app.input[end_idx..]);
                                }
                                app.autocomplete_index = 0;
                                app.show_autocomplete = app.input.starts_with('/');
                            }
                            KeyCode::Char(c) => {
                                let idx = app.input.char_indices()
                                    .nth(app.cursor_pos)
                                    .map(|(i, _)| i)
                                    .unwrap_or(app.input.len());
                                app.input.insert(idx, c);
                                app.cursor_pos += 1;
                                app.autocomplete_index = 0;
                                app.show_autocomplete = app.input.starts_with('/');
                            }
                            _ => {}
                        }
                    } else if app.show_permission_dialog && !app.pending_permissions.is_empty() {
                        match key.code {
                            KeyCode::Left | KeyCode::Char('h') => {
                                if app.permission_selector_index > 0 {
                                    app.permission_selector_index -= 1;
                                }
                            }
                            KeyCode::Right | KeyCode::Char('l') => {
                                if app.permission_selector_index < 2 {
                                    app.permission_selector_index += 1;
                                }
                            }
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.permission_selector_index = 0;
                                // Fall through to Enter handling
                                let perm = app.pending_permissions.remove(0);
                                let server_clone = server.clone();
                                let tx_clone = tx.clone();
                                app.orchestrator_logs.push(format!("[PERMISSION] Approving once: {}", perm.permission));
                                tokio::spawn(async move {
                                    match server_clone.reply_to_permission(&perm.request_id, "once").await {
                                        Ok(()) => {
                                            let _ = tx_clone.send(AppMessage::OrchestratorLog(0, format!("✓ Approved: {}", perm.permission))).await;
                                        }
                                        Err(e) => {
                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to approve: {}", e))).await;
                                        }
                                    }
                                });
                                if app.pending_permissions.is_empty() {
                                    app.show_permission_dialog = false;
                                    app.status = "Ready".to_string();
                                }
                            }
                            KeyCode::Char('a') | KeyCode::Char('A') => {
                                let perm = app.pending_permissions.remove(0);
                                let server_clone = server.clone();
                                let tx_clone = tx.clone();
                                app.orchestrator_logs.push(format!("[PERMISSION] Approving always: {}", perm.permission));
                                tokio::spawn(async move {
                                    match server_clone.reply_to_permission(&perm.request_id, "always").await {
                                        Ok(()) => {
                                            let _ = tx_clone.send(AppMessage::OrchestratorLog(0, format!("✓ Always approved: {}", perm.permission))).await;
                                        }
                                        Err(e) => {
                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to approve: {}", e))).await;
                                        }
                                    }
                                });
                                if app.pending_permissions.is_empty() {
                                    app.show_permission_dialog = false;
                                    app.status = "Ready".to_string();
                                }
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') => {
                                let perm = app.pending_permissions.remove(0);
                                let server_clone = server.clone();
                                let tx_clone = tx.clone();
                                app.orchestrator_logs.push(format!("[PERMISSION] Rejecting: {}", perm.permission));
                                tokio::spawn(async move {
                                    match server_clone.reply_to_permission(&perm.request_id, "reject").await {
                                        Ok(()) => {
                                            let _ = tx_clone.send(AppMessage::OrchestratorLog(0, format!("✗ Rejected: {}", perm.permission))).await;
                                        }
                                        Err(e) => {
                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to reject: {}", e))).await;
                                        }
                                    }
                                });
                                if app.pending_permissions.is_empty() {
                                    app.show_permission_dialog = false;
                                    app.status = "Ready".to_string();
                                }
                            }
                            KeyCode::Enter => {
                                let perm = app.pending_permissions.remove(0);
                                let reply = match app.permission_selector_index {
                                    0 => "once",
                                    1 => "always",
                                    _ => "reject",
                                };
                                let server_clone = server.clone();
                                let tx_clone = tx.clone();
                                let reply_str = reply.to_string();
                                app.orchestrator_logs.push(format!("[PERMISSION] Replying '{}': {}", reply, perm.permission));
                                tokio::spawn(async move {
                                    match server_clone.reply_to_permission(&perm.request_id, &reply_str).await {
                                        Ok(()) => {
                                            let _ = tx_clone.send(AppMessage::OrchestratorLog(0, format!("✓ Permission {}: {}", reply_str, perm.permission))).await;
                                        }
                                        Err(e) => {
                                            let _ = tx_clone.send(AppMessage::Error(format!("Failed to reply: {}", e))).await;
                                        }
                                    }
                                });
                                if app.pending_permissions.is_empty() {
                                    app.show_permission_dialog = false;
                                    app.status = "Ready".to_string();
                                }
                            }
                            KeyCode::Esc => {
                                // Reject on escape
                                let perm = app.pending_permissions.remove(0);
                                let server_clone = server.clone();
                                let tx_clone = tx.clone();
                                app.orchestrator_logs.push(format!("[PERMISSION] Rejecting (Esc): {}", perm.permission));
                                tokio::spawn(async move {
                                    let _ = server_clone.reply_to_permission(&perm.request_id, "reject").await;
                                    let _ = tx_clone.send(AppMessage::OrchestratorLog(0, format!("✗ Rejected: {}", perm.permission))).await;
                                });
                                if app.pending_permissions.is_empty() {
                                    app.show_permission_dialog = false;
                                    app.status = "Ready".to_string();
                                }
                            }
                            _ => {}
                        }
                    } else if app.show_model_selector {
                        match key.code {
                            KeyCode::Up | KeyCode::Char('k') => {
                                if app.model_selector_index > 0 {
                                    app.model_selector_index -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Char('j') => {
                                if app.model_selector_index + 1 < app.model_options.len() {
                                    app.model_selector_index += 1;
                                }
                            }
                            KeyCode::Enter => {
                                if let Some(selected) = app.model_options.get(app.model_selector_index).cloned() {
                                    app.show_model_selector = false;
                                    let server_clone = server.clone();
                                    let tx_clone = tx.clone();
                                    app.status = format!("Setting model to {}/{}...", selected.provider_id, selected.model_id);
                                    tokio::spawn(async move {
                                        match server_clone.set_model(&selected.provider_id, &selected.model_id).await {
                                            Ok(()) => {
                                                let _ = tx_clone.send(AppMessage::CommandResult(format!("Model set to {}/{}", selected.provider_id, selected.model_id))).await;
                                            }
                                            Err(e) => {
                                                let _ = tx_clone.send(AppMessage::Error(format!("Failed to set model: {}", e))).await;
                                            }
                                        }
                                    });
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.show_model_selector = false;
                                app.status = "Model selection cancelled".to_string();
                            }
                            _ => {}
                        }
                    } else if app.confirm_delete {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                let session = app.current_session_mut();
                                if let Some(idx) = session.selected_worker {
                                    let worker_id = session.workers[idx].id;
                                    session.workers.remove(idx);
                                    session.messages.push((format!("🗑 Deleted worker #{}", worker_id), false));
                                    if session.workers.is_empty() {
                                        session.selected_worker = None;
                                    } else if idx >= session.workers.len() {
                                        session.selected_worker = Some(session.workers.len() - 1);
                                    }
                                }
                                app.confirm_delete = false;
                                app.status = "Worker deleted".to_string();
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.confirm_delete = false;
                                app.status = "Delete cancelled".to_string();
                            }
                            _ => {}
                        }
                    } else if app.confirm_clear_all {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                let session = app.current_session_mut();
                                let count = session.workers.len();
                                session.workers.clear();
                                session.selected_worker = None;
                                session.messages.push((format!("🗑 Cleared {} workers", count), false));
                                app.confirm_clear_all = false;
                                app.status = "All workers cleared".to_string();
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.confirm_clear_all = false;
                                app.status = "Clear cancelled".to_string();
                            }
                            _ => {}
                        }
                    } else if app.confirm_delete_session {
                        match key.code {
                            KeyCode::Char('y') | KeyCode::Char('Y') => {
                                app.delete_current_session();
                                app.confirm_delete_session = false;
                            }
                            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                                app.confirm_delete_session = false;
                                app.status = "Delete cancelled".to_string();
                            }
                            _ => {}
                        }
                    } else {
                        let has_selected_worker = app.current_session().selected_worker.is_some();
                        match key.code {
                            KeyCode::Char('q') => app.quit = true,
                            KeyCode::Char('i') | KeyCode::Enter => app.input_mode = true,
                            KeyCode::Char('j') | KeyCode::Down => {
                                if app.show_logs {
                                    app.logs_scroll = app.logs_scroll.saturating_add(1).min(app.orchestrator_logs.len().saturating_sub(1));
                                } else if has_selected_worker {
                                    app.current_session_mut().scroll_offset = app.current_session().scroll_offset.saturating_add(1);
                                } else {
                                    app.current_session_mut().select_next_worker();
                                }
                            }
                            KeyCode::Char('k') | KeyCode::Up => {
                                if app.show_logs {
                                    app.logs_scroll = app.logs_scroll.saturating_sub(1);
                                } else if has_selected_worker {
                                    app.current_session_mut().scroll_offset = app.current_session().scroll_offset.saturating_sub(1);
                                } else {
                                    app.current_session_mut().select_prev_worker();
                                }
                            }
                            KeyCode::Tab => {
                                if has_selected_worker {
                                    app.current_session_mut().select_next_worker();
                                    app.current_session_mut().scroll_offset = 0;
                                } else {
                                    app.next_session();
                                }
                            }
                            KeyCode::BackTab => {
                                if has_selected_worker {
                                    app.current_session_mut().select_prev_worker();
                                    app.current_session_mut().scroll_offset = 0;
                                } else {
                                    app.prev_session();
                                }
                            }
                            KeyCode::Char('n') => app.next_session(),
                            KeyCode::Char('p') => app.prev_session(),
                            KeyCode::Char('d') => {
                                if has_selected_worker {
                                    app.confirm_delete = true;
                                    app.status = "Delete this worker? (y/n)".to_string();
                                }
                            }
                            KeyCode::Char('c') | KeyCode::Char('C') => {
                                if !app.current_session().workers.is_empty() {
                                    app.confirm_clear_all = true;
                                    app.status = format!("Clear all {} workers? (y/n)", app.current_session().workers.len());
                                }
                            }
                            KeyCode::PageDown | KeyCode::Char('J') => {
                                if app.show_logs {
                                    app.logs_scroll = app.logs_scroll.saturating_add(20).min(app.orchestrator_logs.len().saturating_sub(1));
                                } else {
                                    app.current_session_mut().scroll_offset = app.current_session().scroll_offset.saturating_add(10);
                                }
                            }
                            KeyCode::PageUp | KeyCode::Char('K') => {
                                if app.show_logs {
                                    app.logs_scroll = app.logs_scroll.saturating_sub(20);
                                } else {
                                    app.current_session_mut().scroll_offset = app.current_session().scroll_offset.saturating_sub(10);
                                }
                            }
                            KeyCode::Home | KeyCode::Char('g') => {
                                if app.show_logs {
                                    app.logs_scroll = 0;
                                } else {
                                    app.current_session_mut().scroll_offset = 0;
                                }
                            }
                            KeyCode::End | KeyCode::Char('G') => {
                                if app.show_logs {
                                    app.logs_scroll = app.orchestrator_logs.len().saturating_sub(1);
                                } else {
                                    app.current_session_mut().scroll_offset = usize::MAX;
                                }
                            }
                            KeyCode::Char('l') => {
                                app.show_logs = !app.show_logs;
                                if app.show_logs {
                                    // Auto-scroll to bottom when opening logs
                                    app.logs_scroll = app.orchestrator_logs.len().saturating_sub(1);
                                }
                            }
                            KeyCode::Esc => {
                                if app.show_logs {
                                    app.show_logs = false;
                                } else if has_selected_worker {
                                    app.current_session_mut().selected_worker = None;
                                    app.current_session_mut().scroll_offset = 0;
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        while let Ok(msg) = rx.try_recv() {
            match msg {
                AppMessage::OrchestratorLog(_session_id, log) => {
                    app.orchestrator_logs.push(log);
                    app.status = "🤔 Orchestrator analyzing...".to_string();
                }
                AppMessage::ServerLogs(logs) => {
                    for log in logs {
                        if !app.orchestrator_logs.contains(&log) {
                            app.orchestrator_logs.push(log);
                        }
                    }
                }
                AppMessage::TaskPlan(session_id, plan, logs, orch_session_id) => {
                    app.orchestrator_logs.extend(logs);
                    if let Some(session) = app.sessions.iter_mut().find(|s| s.id == session_id) {
                        if session.orchestrator_session_id.is_none() {
                            session.orchestrator_session_id = Some(orch_session_id);
                        }
                        
                        // Clear all workers before spawning new batch
                        session.workers.clear();
                        session.selected_worker = None;
                        
                        session.messages.push((format!("📋 Plan: {}", plan.reasoning), false));
                        session.messages.push((format!("   Spawning {} worker(s)...", plan.tasks.len()), false));
                        
                        for task in &plan.tasks {
                            session.messages.push((format!("   • #{}: {}", task.id, task.description), false));
                            session.workers.push(Worker::new(task.id, task.description.clone()));
                        }
                    }
                    app.status = format!("Running {} workers", plan.tasks.len());
                }
                AppMessage::WorkerStarted(session_id, worker_id, opencode_session_id) => {
                    app.orchestrator_logs.push(format!("[WORKER] Started worker #{} with session {}", worker_id, truncate_str(&opencode_session_id, 12)));
                    if let Some(session) = app.sessions.iter_mut().find(|s| s.id == session_id) {
                        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
                            worker.session_id = Some(opencode_session_id);
                            worker.state = WorkerState::Running;
                            worker.output.push("🚀 Started...".to_string());
                        }
                    }
                }
                AppMessage::WorkerOutput(session_id, worker_id, line) => {
                    if let Some(session) = app.sessions.iter_mut().find(|s| s.id == session_id) {
                        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
                            worker.output.push(line);
                        }
                    }
                }
                AppMessage::WorkerComplete(session_id, worker_id) => {
                    if let Some(session) = app.sessions.iter_mut().find(|s| s.id == session_id) {
                        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
                            worker.state = WorkerState::Complete;
                            worker.output.push("✓ Complete".to_string());
                        }
                        
                        let all_done = session.workers.iter().all(|w| 
                            matches!(w.state, WorkerState::Complete | WorkerState::Error)
                        );
                        if all_done {
                            app.status = "All workers complete - Ready for next task".to_string();
                        }
                    }
                }
                AppMessage::WorkerError(session_id, worker_id, error) => {
                    if let Some(session) = app.sessions.iter_mut().find(|s| s.id == session_id) {
                        if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
                            worker.state = WorkerState::Error;
                            worker.output.push(format!("✗ Error: {}", error));
                        }
                    }
                }
                AppMessage::StreamEvent(event) => {
                    use crate::server::StreamEvent;
                    match event {
                        StreamEvent::Connected => {
                            app.orchestrator_logs.push("[SSE] Connected to event stream".to_string());
                        }
                        StreamEvent::PartUpdated { session_id: opencode_session_id, part } => {
                            if let Some(session) = app.find_session_by_worker_session_id(&opencode_session_id) {
                                if let Some(worker) = session.workers.iter_mut().find(|w| w.session_id.as_ref() == Some(&opencode_session_id)) {
                                    if let Some(text) = &part.text {
                                        worker.streaming_content = text.clone();
                                        worker.current_tool = None;
                                    }
                                }
                                app.orchestrator_logs.push(format!("[SSE] Text for WORKER {}", truncate_str(&opencode_session_id, 16)));
                            } else {
                                app.orchestrator_logs.push(format!("[SSE] Part for unknown {}", truncate_str(&opencode_session_id, 16)));
                            }
                        }
                        StreamEvent::ToolCall { session_id: opencode_session_id, tool_name, status, input } => {
                            let mut question_to_show: Option<(u32, String, String)> = None;
                            
                            if tool_name == "question" {
                                let extracted = extract_question_text(&input);
                                app.orchestrator_logs.push(format!("[QUESTION] status={} extracted='{}'", status, truncate_str(&extracted, 50)));
                            }
                            if let Some(session) = app.find_session_by_worker_session_id(&opencode_session_id) {
                                if let Some(worker) = session.workers.iter_mut().find(|w| w.session_id.as_ref() == Some(&opencode_session_id)) {
                                    let display_name = format_tool_display(&tool_name, &input);
                                    match status.as_str() {
                                        "pending" | "running" => {
                                            worker.current_tool = Some(display_name);
                                            if tool_name == "question" && status == "running" {
                                                let extracted = extract_question_text(&input);
                                                if !extracted.is_empty() {
                                                    worker.pending_question = Some(extracted.clone());
                                                    worker.state = WorkerState::WaitingForInput;
                                                    question_to_show = Some((worker.id, worker.description.clone(), extracted));
                                                }
                                            }
                                        }
                                        "completed" => {
                                            worker.current_tool = None;
                                            if !worker.tool_history.iter().any(|h| h == &display_name) {
                                                worker.tool_history.push(display_name);
                                            }
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            
                            // Show question immediately when detected (outside the borrow)
                            if let Some((worker_id, description, question_text)) = question_to_show {
                                app.current_session_mut().messages.push((format!("❓ Worker #{} ({}) asks:", worker_id, description), false));
                                for line in question_text.lines() {
                                    app.current_session_mut().messages.push((format!("   {}", line), false));
                                }
                                app.current_session_mut().messages.push((format!("   Reply with: /reply #{} <your answer>", worker_id), false));
                                app.current_session_mut().messages.push(("".to_string(), false));
                                app.status = format!("Worker #{} is waiting for your input", worker_id);
                            }
                            
                            app.orchestrator_logs.push(format!("[SSE] Tool {} ({})", tool_name, status));
                        }
                        StreamEvent::SessionIdle { session_id: opencode_session_id } => {
                            app.orchestrator_logs.push(format!("[IDLE-RAW] session={} found_worker={}", 
                                truncate_str(&opencode_session_id, 12),
                                app.sessions.iter().any(|s| s.workers.iter().any(|w| w.session_id.as_ref() == Some(&opencode_session_id)))
                            ));
                            
                            let mut report_data: Option<(usize, String)> = None;
                            let mut question_info: Option<(u32, String, String)> = None;
                            let mut idle_debug: Option<String> = None;
                            
                            if let Some(session) = app.find_session_by_worker_session_id(&opencode_session_id) {
                                let mut worker_report: Option<(u32, String, String)> = None;
                                
                                if let Some(worker) = session.workers.iter_mut().find(|w| w.session_id.as_ref() == Some(&opencode_session_id)) {
                                    idle_debug = Some(format!("[IDLE] worker #{} pending_question={:?}", worker.id, worker.pending_question.as_ref().map(|s| truncate_str(s, 30))));
                                    if let Some(ref question) = worker.pending_question {
                                        worker.state = WorkerState::WaitingForInput;
                                        let question_text = if question.is_empty() {
                                            worker.streaming_content.clone()
                                        } else {
                                            question.clone()
                                        };
                                        question_info = Some((worker.id, worker.description.clone(), question_text));
                                    } else {
                                        worker.state = WorkerState::Complete;
                                        if !worker.streaming_content.is_empty() {
                                            worker.output = worker.streaming_content.lines().map(|s| s.to_string()).collect();
                                        }
                                        worker.output.push("".to_string());
                                        worker.output.push("✓ Complete".to_string());
                                        
                                        let summary = worker.get_summary();
                                        worker_report = Some((worker.id, worker.description.clone(), summary));
                                        worker.streaming_content.clear();
                                    }
                                }
                                
                                if let Some((worker_id, description, summary)) = worker_report {
                                    session.messages.push((format!("━━━ Worker #{}: {} ━━━", worker_id, description), false));
                                    for line in summary.lines() {
                                        session.messages.push((line.to_string(), false));
                                    }
                                    session.messages.push(("".to_string(), false));
                                }
                                
                                let all_done = session.workers.iter().all(|w| 
                                    matches!(w.state, WorkerState::Complete | WorkerState::Error)
                                );
                                if all_done && !session.workers.is_empty() {
                                    let worker_summaries: Vec<String> = session.workers.iter()
                                        .map(|w| format!("Worker #{} ({}): {}", w.id, w.description, w.get_summary()))
                                        .collect();
                                    
                                    session.messages.push(("━━━ All Workers Complete ━━━".to_string(), false));
                                    session.messages.push((format!("{} workers finished", session.workers.len()), false));
                                    session.messages.push(("".to_string(), false));
                                    
                                    let results_report = worker_summaries.join("\n\n");
                                    report_data = Some((session.id, results_report));
                                }
                            }
                            
                            if let Some((worker_id, description, question_text)) = question_info {
                                app.current_session_mut().messages.push((format!("❓ Worker #{} ({}) asks:", worker_id, description), false));
                                for line in question_text.lines() {
                                    app.current_session_mut().messages.push((format!("   {}", line), false));
                                }
                                app.current_session_mut().messages.push((format!("   Reply with: /reply #{} <your answer>", worker_id), false));
                                app.current_session_mut().messages.push(("".to_string(), false));
                                app.status = format!("Worker #{} is waiting for your input", worker_id);
                            } else if let Some((ui_session_id, results_report)) = report_data {
                                app.status = "Reporting results to orchestrator...".to_string();
                                let tx_clone = tx.clone();
                                tokio::spawn(async move {
                                    let _ = tx_clone.send(AppMessage::ReportToOrchestrator(ui_session_id, results_report)).await;
                                });
                            }
                            
                            if let Some(debug) = idle_debug {
                                app.orchestrator_logs.push(debug);
                            }
                            app.orchestrator_logs.push(format!("[SSE] Session idle: {}", truncate_str(&opencode_session_id, 8)));
                        }
                        StreamEvent::QuestionAsked { session_id: opencode_session_id, request_id, questions } => {
                            app.orchestrator_logs.push(format!("[QUESTION.ASKED] request_id={} session={} questions={}", 
                                truncate_str(&request_id, 12),
                                truncate_str(&opencode_session_id, 12),
                                questions.len()
                            ));
                            
                            let mut question_to_show: Option<(u32, String, String)> = None;
                            
                            if let Some(session) = app.find_session_by_worker_session_id(&opencode_session_id) {
                                if let Some(worker) = session.workers.iter_mut().find(|w| w.session_id.as_ref() == Some(&opencode_session_id)) {
                                    let question_texts: Vec<String> = questions.iter()
                                        .map(|q| q.question.clone())
                                        .collect();
                                    let question_text = question_texts.join("\n");
                                    
                                    worker.pending_question = Some(question_text.clone());
                                    worker.pending_question_request_id = Some(request_id.clone());
                                    worker.state = WorkerState::WaitingForInput;
                                    
                                    question_to_show = Some((worker.id, worker.description.clone(), question_text));
                                }
                            }
                            
                            if let Some((worker_id, description, question_text)) = question_to_show {
                                app.current_session_mut().messages.push((format!("❓ Worker #{} ({}) asks:", worker_id, description), false));
                                for line in question_text.lines() {
                                    app.current_session_mut().messages.push((format!("   {}", line), false));
                                }
                                app.current_session_mut().messages.push((format!("   Reply with: /reply #{} <your answer>", worker_id), false));
                                app.current_session_mut().messages.push(("".to_string(), false));
                                app.status = format!("Worker #{} is waiting for your input", worker_id);
                            }
                        }
                        StreamEvent::PermissionAsked { session_id: opencode_session_id, request_id, permission, patterns } => {
                            app.orchestrator_logs.push(format!("[PERMISSION.ASKED] request_id={} session={} permission={} patterns={:?}", 
                                truncate_str(&request_id, 12),
                                truncate_str(&opencode_session_id, 12),
                                permission,
                                patterns
                            ));
                            
                            // Find worker info if this is a worker session
                            let mut worker_info: Option<(u32, String)> = None;
                            for session in &app.sessions {
                                for worker in &session.workers {
                                    if worker.session_id.as_ref() == Some(&opencode_session_id) {
                                        worker_info = Some((worker.id, worker.description.clone()));
                                        break;
                                    }
                                }
                            }
                            
                            // Add to pending permissions
                            let pending = PendingPermission {
                                request_id: request_id.clone(),
                                session_id: opencode_session_id.clone(),
                                permission: permission.clone(),
                                patterns: patterns.clone(),
                                worker_id: worker_info.as_ref().map(|(id, _)| *id),
                                worker_description: worker_info.map(|(_, desc)| desc),
                            };
                            app.pending_permissions.push(pending);
                            
                            // Show permission dialog if not already showing
                            if !app.show_permission_dialog {
                                app.show_permission_dialog = true;
                                app.permission_selector_index = 0;
                            }
                            
                            let worker_str = if let Some(id) = app.pending_permissions.last().and_then(|p| p.worker_id) {
                                format!("Worker #{}", id)
                            } else {
                                "Session".to_string()
                            };
                            app.status = format!("🔐 {} requests {} permission", worker_str, permission);
                        }
                        StreamEvent::Error(e) => {
                            app.orchestrator_logs.push(format!("[SSE] Error: {}", e));
                        }
                    }
                }
                AppMessage::CommandResult(result) => {
                    app.current_session_mut().messages.push((result, false));
                    app.status = "Ready".to_string();
                }
                AppMessage::Error(error) => {
                    app.current_session_mut().messages.push((format!("✗ {}", error), false));
                    app.status = "Error - Ready for next task".to_string();
                }
                AppMessage::ModelsLoaded(options) => {
                    if options.is_empty() {
                        app.current_session_mut().messages.push(("No models available".to_string(), false));
                        app.status = "Ready".to_string();
                    } else {
                        app.model_options = options;
                        app.model_selector_index = 0;
                        app.show_model_selector = true;
                        app.status = "Select a model".to_string();
                    }
                }
                AppMessage::ReportToOrchestrator(ui_session_id, results) => {
                    if let Some(session) = app.sessions.iter().find(|s| s.id == ui_session_id) {
                        if let Some(orch_session_id) = &session.orchestrator_session_id {
                            let server_clone = server.clone();
                            let orch_session_id = orch_session_id.clone();
                            let tx_clone = tx.clone();
                            
                            tokio::spawn(async move {
                                let mut orch = Orchestrator::new(server_clone);
                                orch.set_session_id(orch_session_id);
                                
                                match orch.report_worker_results(&results).await {
                                    Ok(()) => {
                                        let _ = tx_clone.send(AppMessage::CommandResult("Results reported to orchestrator".to_string())).await;
                                    }
                                    Err(e) => {
                                        let _ = tx_clone.send(AppMessage::OrchestratorLog(ui_session_id, format!("Failed to report results: {}", e))).await;
                                        let _ = tx_clone.send(AppMessage::CommandResult("Ready (results report failed)".to_string())).await;
                                    }
                                }
                            });
                        } else {
                            app.status = "All workers complete - Ready for next task".to_string();
                        }
                    } else {
                        app.status = "All workers complete - Ready for next task".to_string();
                    }
                }
            }
        }

        if app.quit {
            break;
        }
    }

    server_process.stop().await?;
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_session_tabs(f, app, chunks[0]);

    if app.show_logs {
        render_logs(f, app, chunks[1]);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        render_workers(f, app, main_chunks[0]);
        render_main_panel(f, app, main_chunks[1]);
    }

    render_input(f, app, chunks[2]);
    render_autocomplete(f, app, chunks[2]);
    render_model_selector(f, app);
    render_permission_dialog(f, app);
    render_status(f, app, chunks[3]);
}

fn render_session_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs: Vec<Span> = app
        .sessions
        .iter()
        .enumerate()
        .flat_map(|(i, session)| {
            let worker_count = session.workers.len();
            let running = session.workers.iter().filter(|w| matches!(w.state, WorkerState::Running | WorkerState::Starting)).count();
            
            let indicator = if running > 0 {
                format!(" ◐{}", running)
            } else if worker_count > 0 {
                format!(" ●{}", worker_count)
            } else {
                String::new()
            };
            
            let label = format!(" {}{} ", session.name, indicator);
            
            let style = if i == app.current_session {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::Gray)
            };
            
            vec![
                Span::styled(label, style),
                Span::raw(" "),
            ]
        })
        .collect();

    let line = Line::from(tabs);
    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}

fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let total_logs = app.orchestrator_logs.len();
    
    let lines: Vec<Line> = app
        .orchestrator_logs
        .iter()
        .map(|log| {
            let style = if log.contains("Success") || log.contains("[SPAWN]") || log.contains("[WORKER]") {
                Style::default().fg(Color::Green)
            } else if log.contains("Failed") || log.contains("error") || log.contains("Error") {
                Style::default().fg(Color::Red)
            } else if log.contains("Attempt") || log.contains("[QUESTION]") || log.contains("[IDLE") {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::styled(log.as_str(), style)
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" Orchestrator Logs ({}) - scroll or l to close ", total_logs))
                .title_style(Style::default().fg(Color::Magenta).bold())
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .scroll((app.logs_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

fn render_workers(f: &mut Frame, app: &App, area: Rect) {
    let session = app.current_session();
    let items: Vec<ListItem> = session
        .workers
        .iter()
        .enumerate()
        .map(|(i, worker)| {
            let (icon, _color) = match worker.state {
                WorkerState::Starting => ("◌", Color::Gray),
                WorkerState::Running => ("◐", Color::Yellow),
                WorkerState::WaitingForInput => ("❓", Color::Magenta),
                WorkerState::Complete => ("●", Color::Green),
                WorkerState::Error => ("✗", Color::Red),
            };

            let style = if Some(i) == session.selected_worker {
                Style::default().fg(Color::Cyan).bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };

            let text = format!("{} #{} {}", icon, worker.id, truncate_str(&worker.description, 17));

            ListItem::new(Line::styled(text, style))
        })
        .collect();

    let title = if session.workers.is_empty() {
        " Workers (none) ".to_string()
    } else {
        let _running = session.workers.iter().filter(|w| matches!(w.state, WorkerState::Running | WorkerState::Starting)).count();
        let done = session.workers.iter().filter(|w| matches!(w.state, WorkerState::Complete)).count();
        format!(" Workers ({}/{} done) ", done, session.workers.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(title)
                .title_style(Style::default().fg(Color::Yellow).bold())
                .border_style(Style::default().fg(Color::Yellow)),
        );

    f.render_widget(list, area);
}

fn render_main_panel(f: &mut Frame, app: &App, area: Rect) {
    let session = app.current_session();
    
    if let Some(idx) = session.selected_worker {
        if let Some(worker) = session.workers.get(idx) {
            let display_lines = worker.get_display_lines();
            let output_height = area.height.saturating_sub(2) as usize;
            
            let total_lines = display_lines.len();
            let max_scroll = total_lines.saturating_sub(output_height);
            let auto_scroll = worker.state == WorkerState::Running;
            let scroll = if auto_scroll {
                max_scroll
            } else {
                session.scroll_offset.min(max_scroll)
            };
            
            let lines: Vec<Line> = display_lines
                .iter()
                .skip(scroll)
                .take(output_height)
                .map(|line| {
                    if line.starts_with("✓") {
                        Line::styled(line.as_str(), Style::default().fg(Color::Green))
                    } else if line.starts_with("✗") {
                        Line::styled(line.as_str(), Style::default().fg(Color::Red))
                    } else if line.starts_with("⚙") {
                        Line::styled(line.as_str(), Style::default().fg(Color::Yellow))
                    } else if line.starts_with("🚀") {
                        Line::styled(line.as_str(), Style::default().fg(Color::Yellow))
                    } else {
                        Line::from(line.as_str())
                    }
                })
                .collect();

            let state_str = match worker.state {
                WorkerState::Starting => "Starting",
                WorkerState::Running => "Streaming...",
                WorkerState::WaitingForInput => "Waiting for input",
                WorkerState::Complete => "Complete",
                WorkerState::Error => "Error",
            };

            let scroll_info = if total_lines > output_height {
                format!(" [{}/{}] ", scroll + output_height.min(total_lines), total_lines)
            } else {
                String::new()
            };

            let paragraph = Paragraph::new(lines)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded)
                        .title(format!(" Worker #{} - {} [{}]{}", worker.id, truncate_str(&worker.description, 20), state_str, scroll_info))
                        .title_style(Style::default().fg(Color::Cyan).bold())
                        .border_style(Style::default().fg(Color::Cyan)),
                )
                .wrap(Wrap { trim: false });

            f.render_widget(paragraph, area);
            return;
        }
    }

    let lines: Vec<Line> = session
        .messages
        .iter()
        .map(|(msg, is_user)| {
            if *is_user {
                Line::styled(msg.as_str(), Style::default().fg(Color::Magenta).bold())
            } else if msg.starts_with("📋") {
                Line::styled(msg.as_str(), Style::default().fg(Color::Green))
            } else if msg.starts_with("✗") {
                Line::styled(msg.as_str(), Style::default().fg(Color::Red))
            } else {
                Line::styled(msg.as_str(), Style::default().fg(Color::Gray))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" {} ", session.name))
                .title_style(Style::default().fg(Color::Cyan).bold())
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let style = if app.input_mode {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let spans = if app.input_mode {
        let chars: Vec<char> = app.input.chars().collect();
        let before: String = chars[..app.cursor_pos].iter().collect();
        let cursor_char = chars.get(app.cursor_pos).copied().unwrap_or(' ');
        let after: String = if app.cursor_pos < chars.len() {
            chars[app.cursor_pos + 1..].iter().collect()
        } else {
            String::new()
        };
        
        vec![
            Span::styled("› ", Style::default().fg(Color::Cyan).bold()),
            Span::raw(before),
            Span::styled(cursor_char.to_string(), Style::default().fg(Color::Black).bg(Color::Yellow)),
            Span::raw(after),
        ]
    } else {
        vec![
            Span::styled("› ", Style::default().fg(Color::Cyan).bold()),
            Span::raw(&app.input),
        ]
    };

    let input = Paragraph::new(Line::from(spans))
        .style(style)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(if app.input_mode { " Type your task (Enter to send, Esc to navigate) " } else { " Press 'i' to type " })
                .title_style(Style::default().fg(if app.input_mode { Color::Yellow } else { Color::DarkGray }))
                .border_style(Style::default().fg(if app.input_mode { Color::Yellow } else { Color::DarkGray })),
        );

    f.render_widget(input, area);
}

fn render_permission_dialog(f: &mut Frame, app: &App) {
    if !app.show_permission_dialog || app.pending_permissions.is_empty() {
        return;
    }

    let perm = &app.pending_permissions[0];
    let area = f.area();
    let popup_width = 70u16.min(area.width.saturating_sub(4));
    let popup_height = 12u16.min(area.height.saturating_sub(4));
    
    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let worker_str = if let Some(id) = perm.worker_id {
        format!("Worker #{}", id)
    } else {
        "Session".to_string()
    };
    
    let desc_str = perm.worker_description.as_ref()
        .map(|d| format!(" ({})", d))
        .unwrap_or_default();

    let patterns_display: Vec<String> = perm.patterns.iter()
        .map(|p| truncate_str(p, 60))
        .collect();

    let mut lines = vec![
        Line::from(vec![
            Span::styled("Permission Request", Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("From: "),
            Span::styled(format!("{}{}", worker_str, desc_str), Style::default().fg(Color::Cyan)),
        ]),
        Line::from(vec![
            Span::raw("Tool: "),
            Span::styled(&perm.permission, Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("Files: "),
        ]),
    ];
    
    for pattern in &patterns_display {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(pattern, Style::default().fg(Color::White)),
        ]));
    }
    
    lines.push(Line::from(""));
    
    let options = vec![
        ("y", "Yes (once)", app.permission_selector_index == 0),
        ("a", "Always", app.permission_selector_index == 1),
        ("n", "No (reject)", app.permission_selector_index == 2),
    ];
    
    let option_spans: Vec<Span> = options.iter()
        .enumerate()
        .flat_map(|(i, (key, label, selected))| {
            let style = if *selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::Gray)
            };
            let sep = if i < options.len() - 1 { "  " } else { "" };
            vec![
                Span::styled(format!("[{}] {}", key, label), style),
                Span::raw(sep),
            ]
        })
        .collect();
    
    lines.push(Line::from(option_spans));

    let pending_count = app.pending_permissions.len();
    if pending_count > 1 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![
            Span::styled(format!("({} more pending)", pending_count - 1), Style::default().fg(Color::DarkGray)),
        ]));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title(" 🔐 Permission Required ")
                .title_style(Style::default().fg(Color::Yellow).bold())
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

fn render_model_selector(f: &mut Frame, app: &App) {
    if !app.show_model_selector {
        return;
    }

    let area = f.area();
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_height = (app.model_options.len() + 2).min(20) as u16;
    
    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let items: Vec<ListItem> = app.model_options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let style = if i == app.model_selector_index {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let provider_display = if opt.provider_name != opt.provider_id {
                &opt.provider_name
            } else {
                &opt.provider_id
            };
            let display = if opt.model_name != opt.model_id {
                format!("{}: {} ({})", provider_display, opt.model_name, opt.model_id)
            } else {
                format!("{}: {}", provider_display, opt.model_id)
            };
            ListItem::new(Line::styled(display, style))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Select Model (↑↓/jk Enter Esc) ")
                .title_style(Style::default().fg(Color::Cyan).bold())
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(list, popup_area);
}

fn render_autocomplete(f: &mut Frame, app: &App, input_area: Rect) {
    if !app.show_autocomplete || !app.input_mode {
        return;
    }

    let suggestions = app.get_current_suggestions();
    if suggestions.is_empty() {
        return;
    }

    let popup_height = (suggestions.len() + 2).min(8) as u16;
    let popup_width = 45u16;
    
    let popup_area = Rect {
        x: input_area.x + 2,
        y: input_area.y.saturating_sub(popup_height),
        width: popup_width.min(input_area.width.saturating_sub(4)),
        height: popup_height,
    };

    let items: Vec<ListItem> = suggestions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.autocomplete_index {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };
            let text = format!("{:<20} {}", s.command, s.description);
            ListItem::new(Line::styled(text, style))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" Commands (Tab/↑↓ to select) ")
                .title_style(Style::default().fg(Color::Cyan).bold())
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(ratatui::widgets::Clear, popup_area);
    f.render_widget(list, popup_area);
}

fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let session = app.current_session();
    let status_color = if app.confirm_delete || app.confirm_clear_all || app.confirm_delete_session {
        Color::Red
    } else if app.status.contains("Error") {
        Color::Red
    } else if app.status.contains("Running") || app.status.contains("analyzing") {
        Color::Yellow
    } else {
        Color::Green
    };

    let keys = if app.confirm_delete || app.confirm_clear_all || app.confirm_delete_session {
        "y: Yes | n: No, cancel"
    } else if app.input_mode {
        "Enter: Send | Esc: Navigate"
    } else if app.show_logs {
        "l: Close logs | n/p: Session | q: Quit"
    } else if session.selected_worker.is_some() {
        "j/k: Scroll | Tab: Next worker | Esc: Back | g/G: Top/Bottom | q: Quit"
    } else if !session.workers.is_empty() {
        "j/k: Select worker | n/p: Session | l: Logs | q: Quit"
    } else {
        "n/p: Session | i: Input | l: Logs | q: Quit"
    };

    let border_color = if app.confirm_delete || app.confirm_clear_all || app.confirm_delete_session { Color::Red } else { Color::DarkGray };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(&app.status, Style::default().fg(status_color).bold()),
        Span::raw("  │  "),
        Span::styled(keys, Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color)),
    );

    f.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_question_text_single() {
        let input: serde_json::Value = serde_json::json!({
            "question": "What file do you want?"
        });
        assert_eq!(extract_question_text(&input), "What file do you want?");
    }

    #[test]
    fn test_extract_question_text_array() {
        let input: serde_json::Value = serde_json::json!({
            "questions": [
                {"header": "File names", "question": "What should the 3 files be named?"},
                {"header": "File type", "question": "What type of files?"}
            ]
        });
        assert_eq!(extract_question_text(&input), "What should the 3 files be named?\nWhat type of files?");
    }

    #[test]
    fn test_extract_question_text_empty() {
        let input: serde_json::Value = serde_json::json!({});
        assert_eq!(extract_question_text(&input), "");
    }

    #[test]
    fn test_extract_question_text_string_input() {
        // Test when input is a JSON string that needs parsing
        let input: serde_json::Value = serde_json::json!(r#"{"questions":[{"header":"Test","question":"What is your name?"}]}"#);
        assert_eq!(extract_question_text(&input), "What is your name?");
    }
}

//! Event handlers for the TUI.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseButton, MouseEvent, MouseEventKind};
use tokio::sync::mpsc::Sender;

use crate::constants::{LINE_SCROLL_AMOUNT, PAGE_SCROLL_LINES};
use crate::orchestrator::Orchestrator;
use crate::server::{OpenCodeServer, StreamEvent};
use crate::utils::{extract_question_text, format_tool_display};

use super::app::App;
use super::commands::{parse_slash_command, SlashCommand};
use super::messages::{AppMessage, ModelOption, PendingPermission};
use super::scroll::ScrollDirection;
use super::textarea::TextAreaAction;
use super::worker::{Worker, WorkerState};

/// Handle mouse events for scrolling and text selection.
/// Returns true if a copy operation was performed.
pub fn handle_mouse_event(app: &mut App, mouse: MouseEvent) -> bool {
    match mouse.kind {
        MouseEventKind::ScrollDown => {
            app.handle_scroll(ScrollDirection::Down);
            false
        }
        MouseEventKind::ScrollUp => {
            app.handle_scroll(ScrollDirection::Up);
            false
        }
        MouseEventKind::ScrollLeft => {
            app.handle_scroll(ScrollDirection::Left);
            false
        }
        MouseEventKind::ScrollRight => {
            app.handle_scroll(ScrollDirection::Right);
            false
        }
        MouseEventKind::Down(MouseButton::Left) => {
            app.start_selection(mouse.column, mouse.row);
            false
        }
        MouseEventKind::Drag(MouseButton::Left) => {
            app.update_selection(mouse.column, mouse.row);
            false
        }
        MouseEventKind::Up(MouseButton::Left) => {
            app.finish_selection();
            // Auto-copy on selection (like opencode)
            if let Some(text) = app.get_selected_text() {
                if !text.is_empty() {
                    if let Ok(mut ctx) = arboard::Clipboard::new() {
                        let _ = ctx.set_text(&text);
                        // Also write OSC 52 for SSH/tmux support
                        write_osc52(&text);
                        app.status = format!("Copied {} chars", text.len());
                        return true;
                    }
                }
            }
            false
        }
        _ => false,
    }
}

/// Write text to clipboard via OSC 52 escape sequence.
/// This allows clipboard operations to work over SSH by having
/// the terminal emulator handle the clipboard locally.
fn write_osc52(text: &str) {
    use std::io::Write;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    
    let base64 = STANDARD.encode(text);
    let osc52 = format!("\x1b]52;c;{}\x07", base64);
    
    // Check if we're in tmux or screen
    let passthrough = std::env::var("TMUX").is_ok() || std::env::var("STY").is_ok();
    let sequence = if passthrough {
        format!("\x1bPtmux;\x1b{}\x1b\\", osc52)
    } else {
        osc52
    };
    
    let _ = std::io::stdout().write_all(sequence.as_bytes());
    let _ = std::io::stdout().flush();
}

pub async fn handle_key_event(
    app: &mut App,
    key: KeyEvent,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
    if key.code == KeyCode::Char('c')
        && key.modifiers.contains(KeyModifiers::CONTROL)
        && app.selection.is_some()
    {
        if let Some(text) = app.copy_selection() {
            if let Ok(mut ctx) = arboard::Clipboard::new() {
                let _ = ctx.set_text(&text);
                app.status = format!("Copied {} chars", text.len());
            }
        }
        return;
    }

    if app.input_mode {
        handle_input_mode(app, key, server, tx).await;
    } else if app.show_permission_dialog && !app.pending_permissions.is_empty() {
        handle_permission_dialog(app, key, server, tx).await;
    } else if app.show_model_selector {
        handle_model_selector(app, key, server, tx).await;
    } else if app.show_stop_selector {
        handle_stop_selector(app, key);
    } else if app.confirm_delete {
        handle_confirm_delete(app, key);
    } else if app.confirm_clear_all {
        handle_confirm_clear_all(app, key);
    } else if app.confirm_delete_session {
        handle_confirm_delete_session(app, key);
    } else {
        handle_navigation_mode(app, key);
    }
}

pub async fn handle_app_message(
    app: &mut App,
    msg: AppMessage,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
    match msg {
        AppMessage::OrchestratorLog(_session_id, log) => {
            app.orchestrator_logs.push(log);
            app.status = "Orchestrator analyzing...".to_string();
        }
        AppMessage::ServerLogs(logs) => {
            for log in logs {
                if !app.orchestrator_logs.contains(&log) {
                    app.orchestrator_logs.push(log);
                }
            }
        }
        AppMessage::TaskPlan(session_id, plan, logs, orch_session_id) => {
            handle_task_plan(app, session_id, plan, logs, orch_session_id);
        }
        AppMessage::WorkerStarted(session_id, worker_id, opencode_session_id) => {
            handle_worker_started(app, session_id, worker_id, opencode_session_id);
        }
        AppMessage::WorkerOutput(session_id, worker_id, line) => {
            app.update_worker(session_id, worker_id, |worker| {
                worker.output.push(line);
            });
        }
        AppMessage::WorkerComplete(session_id, worker_id) => {
            handle_worker_complete(app, session_id, worker_id);
        }
        AppMessage::WorkerError(session_id, worker_id, error) => {
            app.update_worker(session_id, worker_id, |worker| {
                worker.state = WorkerState::Error;
                worker.output.push(format!("Error: {}", error));
            });
        }
        AppMessage::StreamEvent(event) => {
            handle_stream_event(app, event, server, tx).await;
        }
        AppMessage::CommandResult(result) => {
            app.current_session_mut().messages.push((result, false));
            app.status = "Ready".to_string();
        }
        AppMessage::Error(error) => {
            app.current_session_mut()
                .messages
                .push((format!("Error: {}", error), false));
            app.status = "Error - Ready for next task".to_string();
        }
        AppMessage::ModelsLoaded(options) => {
            if options.is_empty() {
                app.current_session_mut()
                    .messages
                    .push(("No models available".to_string(), false));
                app.status = "Ready".to_string();
            } else {
                app.model_options = options;
                app.model_selector_index = 0;
                app.show_model_selector = true;
                app.status = "Select a model".to_string();
            }
        }
        AppMessage::ReportToOrchestrator(ui_session_id, results) => {
            handle_report_to_orchestrator(app, ui_session_id, results, server, tx).await;
        }
        AppMessage::CurrentModelLoaded(model) => {
            app.set_current_model(model);
        }
    }
}

async fn handle_input_mode(
    app: &mut App,
    key: KeyEvent,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
    if app.show_autocomplete {
        match key.code {
            KeyCode::Esc => {
                app.show_autocomplete = false;
                return;
            }
            KeyCode::Down => {
                app.autocomplete_next();
                return;
            }
            KeyCode::Up => {
                app.autocomplete_prev();
                return;
            }
            KeyCode::Tab | KeyCode::Enter => {
                if !app.get_current_suggestions().is_empty() {
                    app.apply_autocomplete();
                    return;
                }
            }
            _ => {}
        }
    }

    let action = app.textarea.handle_key(key);

    match action {
        TextAreaAction::Submit => {
            if !app.input_is_empty() {
                let message = app.submit_input();
                app.show_autocomplete = false;
                app.autocomplete_index = 0;
                handle_submit_input(app, message, server, tx).await;
            }
        }
        TextAreaAction::Escape => {
            app.input_mode = false;
        }
        TextAreaAction::Clear => {
            app.clear_input();
            app.show_autocomplete = false;
            app.autocomplete_index = 0;
        }
        TextAreaAction::HistoryPrevious => {
            app.textarea.history_previous();
        }
        TextAreaAction::HistoryNext => {
            app.textarea.history_next();
        }
        TextAreaAction::Continue => {
            app.autocomplete_index = 0;
            app.show_autocomplete = app.input_starts_with("/");
            if app.show_autocomplete && key.code == KeyCode::Tab {
                app.autocomplete_index = 0;
            }
        }
    }
}

async fn handle_submit_input(
    app: &mut App,
    message: String,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
    if let Some(cmd) = parse_slash_command(&message) {
        app.current_session_mut()
            .messages
            .push((format!("> {}", message), true));
        handle_slash_command(app, cmd, server, tx).await;
    } else {
        let session_id = app.current_session().id;
        let existing_orch_session = app.current_session().orchestrator_session_id.clone();
        let current_model = app.current_model.clone();
        app.current_session_mut()
            .messages
            .push((format!("> {}", message), true));
        app.status = "Orchestrator analyzing...".to_string();

        let server_clone = server.clone();
        let tx_clone = tx.clone();
        let msg = message.clone();

        tokio::spawn(async move {
            let _ = tx_clone
                .send(AppMessage::OrchestratorLog(
                    session_id,
                    "Starting orchestrator...".to_string(),
                ))
                .await;

            let mut orch = Orchestrator::new(server_clone.clone());
            orch.set_model(current_model.clone());

            if let Some(orch_session_id) = existing_orch_session {
                orch.set_session_id(orch_session_id);
            } else if let Err(e) = orch.init().await {
                for log in orch.get_logs() {
                    let _ = tx_clone
                        .send(AppMessage::OrchestratorLog(session_id, log.clone()))
                        .await;
                }
                let _ = tx_clone
                    .send(AppMessage::Error(format!(
                        "Orchestrator init failed: {}",
                        e
                    )))
                    .await;
                return;
            }

            for log in orch.get_logs() {
                let _ = tx_clone
                    .send(AppMessage::OrchestratorLog(session_id, log.clone()))
                    .await;
            }

            let orch_session_id = orch.get_session_id().cloned().unwrap_or_default();

            match orch.plan_tasks(&msg).await {
                Ok(plan) => {
                    let logs = orch.get_logs().to_vec();
                    let _ = tx_clone
                        .send(AppMessage::TaskPlan(
                            session_id,
                            plan.clone(),
                            logs,
                            orch_session_id,
                        ))
                        .await;

                    for task in plan.tasks {
                        let server = server_clone.clone();
                        let tx = tx_clone.clone();
                        let task_id = task.id;
                        let prompt = task.prompt.clone();
                        let model = current_model.clone();

                        tokio::spawn(async move {
                            let _ = tx
                                .send(AppMessage::WorkerOutput(
                                    session_id,
                                    task_id,
                                    "Creating session...".to_string(),
                                ))
                                .await;

                            match server
                                .create_session(Some(&format!("Worker {}", task_id)))
                                .await
                            {
                                Ok(session) => {
                                    let _ = tx
                                        .send(AppMessage::WorkerStarted(
                                            session_id,
                                            task_id,
                                            session.id.clone(),
                                        ))
                                        .await;
                                    let _ = tx
                                        .send(AppMessage::WorkerOutput(
                                            session_id,
                                            task_id,
                                            format!(
                                                "Sending to model: {}...",
                                                model.as_deref().unwrap_or("default")
                                            ),
                                        ))
                                        .await;

                                    if let Err(e) = server
                                        .send_message_async_with_model(
                                            &session.id,
                                            &prompt,
                                            model.as_deref(),
                                        )
                                        .await
                                    {
                                        let _ = tx
                                            .send(AppMessage::WorkerError(
                                                session_id,
                                                task_id,
                                                format!("Send failed: {}", e),
                                            ))
                                            .await;
                                    }
                                }
                                Err(e) => {
                                    let _ = tx
                                        .send(AppMessage::WorkerError(
                                            session_id,
                                            task_id,
                                            format!("Create session failed: {}", e),
                                        ))
                                        .await;
                                }
                            }
                        });
                    }
                }
                Err(e) => {
                    for log in orch.get_logs() {
                        let _ = tx_clone
                            .send(AppMessage::OrchestratorLog(session_id, log.clone()))
                            .await;
                    }
                    let _ = tx_clone
                        .send(AppMessage::Error(format!("Planning failed: {}", e)))
                        .await;
                }
            }
        });
    }
}

async fn handle_slash_command(
    app: &mut App,
    cmd: SlashCommand,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
    match cmd {
        SlashCommand::Help => {
            let session = app.current_session_mut();
            session.messages.push(("Commands:".to_string(), false));
            session
                .messages
                .push(("  /new [name]    - Create session".to_string(), false));
            session
                .messages
                .push(("  /sessions      - List sessions".to_string(), false));
            session
                .messages
                .push(("  /rename <name> - Rename session".to_string(), false));
            session
                .messages
                .push(("  /delete        - Delete session".to_string(), false));
            session
                .messages
                .push(("  /models        - List models".to_string(), false));
            session
                .messages
                .push(("  /model         - Select model".to_string(), false));
            session
                .messages
                .push(("  /reply #N [msg] - Reply/continue worker".to_string(), false));
            session
                .messages
                .push(("  /stop          - Stop running workers".to_string(), false));
            session
                .messages
                .push(("  /clear         - Clear messages".to_string(), false));
        }
        SlashCommand::NewSession(name) => {
            app.create_session(name);
        }
        SlashCommand::ListSessions => {
            let list: Vec<String> = app
                .sessions
                .iter()
                .enumerate()
                .map(|(i, s)| {
                    let marker = if i == app.current_session { ">" } else { " " };
                    format!(
                        "{} {}: {} ({} workers)",
                        marker,
                        i + 1,
                        s.name,
                        s.workers.len()
                    )
                })
                .collect();
            let session = app.current_session_mut();
            session.messages.push(("Sessions:".to_string(), false));
            for line in list {
                session.messages.push((format!("  {}", line), false));
            }
        }
        SlashCommand::RenameSession(name) => {
            app.current_session_mut().name = name.clone();
            app.status = format!("Renamed to '{}'", name);
        }
        SlashCommand::DeleteSession => {
            if app.sessions.len() <= 1 {
                app.current_session_mut()
                    .messages
                    .push(("Cannot delete only session".to_string(), false));
            } else {
                app.confirm_delete_session = true;
                app.status = format!("Delete '{}'? (y/n)", app.current_session().name);
            }
        }
        SlashCommand::Clear => {
            app.current_session_mut().messages.clear();
            app.current_session_mut()
                .messages
                .push(("Cleared".to_string(), false));
        }
        SlashCommand::Models => {
            let server_clone = server.clone();
            let tx_clone = tx.clone();
            app.status = "Fetching models...".to_string();
            tokio::spawn(async move {
                match server_clone.get_providers().await {
                    Ok(resp) => {
                        let _ = tx_clone
                            .send(AppMessage::CommandResult(format!(
                                "Providers: {}",
                                resp.connected.join(", ")
                            )))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppMessage::Error(format!("Failed: {}", e)))
                            .await;
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
                    Ok(resp) => {
                        let mut options = Vec::new();
                        for provider in &resp.all {
                            if resp.connected.contains(&provider.id) {
                                let provider_name =
                                    provider.name.as_ref().unwrap_or(&provider.id).clone();
                                for model in provider.models.values() {
                                    let model_name =
                                        model.name.as_ref().unwrap_or(&model.id).clone();
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
                        let _ = tx_clone
                            .send(AppMessage::Error(format!("Failed: {}", e)))
                            .await;
                    }
                }
            });
        }
        SlashCommand::ModelSet(provider, model) => {
            let server_clone = server.clone();
            let tx_clone = tx.clone();
            let model_str = format!("{}/{}", provider, model);
            app.status = format!("Setting {}...", model_str);
            tokio::spawn(async move {
                match server_clone.set_model(&provider, &model).await {
                    Ok(()) => {
                        // Update the displayed model
                        let _ = tx_clone
                            .send(AppMessage::CurrentModelLoaded(Some(model_str.clone())))
                            .await;
                        let _ = tx_clone
                            .send(AppMessage::CommandResult(format!(
                                "Model set to {}",
                                model_str
                            )))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppMessage::Error(format!("Failed: {}", e)))
                            .await;
                    }
                }
            });
        }
        SlashCommand::Reply(worker_id, reply_message) => {
            let session = app.current_session_mut();
            if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
                if worker.state == WorkerState::WaitingForInput {
                    if let Some(request_id) = worker.pending_question_request_id.clone() {
                        let msg = reply_message.clone().unwrap_or_default();
                        worker.state = WorkerState::Running;
                        worker.pending_question = None;
                        worker.pending_question_request_id = None;
                        session
                            .messages
                            .push((format!("[To #{}] {}", worker_id, msg), true));

                        let server_clone = server.clone();
                        let tx_clone = tx.clone();
                        tokio::spawn(async move {
                            let answers = vec![vec![msg]];
                            if let Err(e) =
                                server_clone.reply_to_question(&request_id, answers).await
                            {
                                let _ = tx_clone
                                    .send(AppMessage::Error(format!("Reply failed: {}", e)))
                                    .await;
                            }
                        });
                    }
                } else if let Some(opencode_session_id) = worker.session_id.clone() {
                    if let Some(msg) = reply_message.clone() {
                        worker.state = WorkerState::Running;
                        worker.streaming_content.clear();
                        worker.current_tool = None;
                        session
                            .messages
                            .push((format!("[To #{}] {}", worker_id, msg), true));

                        let server_clone = server.clone();
                        let tx_clone = tx.clone();
                        let current_model = app.current_model.clone();
                        tokio::spawn(async move {
                            if let Err(e) = server_clone
                                .send_message_async_with_model(
                                    &opencode_session_id,
                                    &msg,
                                    current_model.as_deref(),
                                )
                                .await
                            {
                                let _ = tx_clone
                                    .send(AppMessage::Error(format!("Continue failed: {}", e)))
                                    .await;
                            }
                        });
                    } else {
                        session.messages.push((
                            format!("Worker #{} requires a message to continue", worker_id),
                            false,
                        ));
                    }
                } else {
                    session.messages.push((
                        format!("Worker #{} has no session to continue", worker_id),
                        false,
                    ));
                }
            } else {
                session
                    .messages
                    .push((format!("Worker #{} not found", worker_id), false));
            }
        }
        SlashCommand::Projects => {
            let server_clone = server.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                match server_clone.list_projects().await {
                    Ok(projects) => {
                        for p in projects {
                            let _ = tx_clone
                                .send(AppMessage::CommandResult(format!("  {}", p.worktree)))
                                .await;
                        }
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppMessage::Error(format!("Failed: {}", e)))
                            .await;
                    }
                }
            });
        }
        SlashCommand::ProjectCurrent => {
            let server_clone = server.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                match server_clone.get_current_project().await {
                    Ok(p) => {
                        let _ = tx_clone
                            .send(AppMessage::CommandResult(format!(
                                "Current: {}",
                                p.worktree
                            )))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppMessage::Error(format!("Failed: {}", e)))
                            .await;
                    }
                }
            });
        }
        SlashCommand::Path => {
            let server_clone = server.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                match server_clone.get_path().await {
                    Ok(path) => {
                        let _ = tx_clone
                            .send(AppMessage::CommandResult(format!("Path: {}", path)))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppMessage::Error(format!("Failed: {}", e)))
                            .await;
                    }
                }
            });
        }
        SlashCommand::Config => {
            let server_clone = server.clone();
            let tx_clone = tx.clone();
            tokio::spawn(async move {
                match server_clone.get_config().await {
                    Ok(config) => {
                        // Format config as readable output
                        let formatted = serde_json::to_string_pretty(&config)
                            .unwrap_or_else(|_| format!("{:?}", config));
                        let _ = tx_clone
                            .send(AppMessage::CommandResult(format!("Config:\n{}", formatted)))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppMessage::Error(format!("Failed to get config: {}", e)))
                            .await;
                    }
                }
            });
        }
        SlashCommand::Unknown(cmd) => {
            app.current_session_mut()
                .messages
                .push((format!("Unknown: /{}", cmd), false));
        }
        SlashCommand::Stop => {
            let running_count = app.get_running_workers().len();
            if running_count == 0 {
                app.current_session_mut()
                    .messages
                    .push(("No running workers to stop".to_string(), false));
                app.status = "No running workers".to_string();
            } else {
                app.show_stop_selector = true;
                app.stop_selector_cursor = 0;
                app.stop_selector_selections.clear();
                app.status = format!("Select workers to stop ({} running)", running_count);
            }
        }
    }
}

async fn handle_permission_dialog(
    app: &mut App,
    key: KeyEvent,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
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
            reply_permission(app, server, tx, "once").await;
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            reply_permission(app, server, tx, "always").await;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            reply_permission(app, server, tx, "reject").await;
        }
        KeyCode::Enter => {
            let reply = match app.permission_selector_index {
                0 => "once",
                1 => "always",
                _ => "reject",
            };
            reply_permission(app, server, tx, reply).await;
        }
        _ => {}
    }
}

async fn reply_permission(
    app: &mut App,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
    reply: &str,
) {
    if app.pending_permissions.is_empty() {
        return;
    }
    let perm = app.pending_permissions.remove(0);
    let server_clone = server.clone();
    let tx_clone = tx.clone();
    let reply_str = reply.to_string();
    let perm_name = perm.permission.clone();

    tokio::spawn(async move {
        if let Err(e) = server_clone
            .reply_to_permission(&perm.request_id, &reply_str)
            .await
        {
            let _ = tx_clone
                .send(AppMessage::Error(format!("Permission reply failed: {}", e)))
                .await;
        }
    });

    app.orchestrator_logs
        .push(format!("[PERM] {} -> {}", perm_name, reply));

    if app.pending_permissions.is_empty() {
        app.show_permission_dialog = false;
        app.status = "Ready".to_string();
    }
}

async fn handle_model_selector(
    app: &mut App,
    key: KeyEvent,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
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
                let model_str = format!("{}/{}", selected.provider_id, selected.model_id);
                app.status = format!("Setting {}...", model_str);
                tokio::spawn(async move {
                    match server_clone
                        .set_model(&selected.provider_id, &selected.model_id)
                        .await
                    {
                        Ok(()) => {
                            let _ = tx_clone
                                .send(AppMessage::CurrentModelLoaded(Some(model_str.clone())))
                                .await;
                            let _ = tx_clone
                                .send(AppMessage::CommandResult(format!("Model: {}", model_str)))
                                .await;
                        }
                        Err(e) => {
                            let _ = tx_clone
                                .send(AppMessage::Error(format!("Failed: {}", e)))
                                .await;
                        }
                    }
                });
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.show_model_selector = false;
            app.status = "Cancelled".to_string();
        }
        _ => {}
    }
}

fn handle_confirm_delete(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let session = app.current_session_mut();
            if let Some(idx) = session.selected_worker {
                let id = session.workers[idx].id;
                session.workers.remove(idx);
                session
                    .messages
                    .push((format!("Deleted worker #{}", id), false));
                if session.workers.is_empty() {
                    session.selected_worker = None;
                } else if idx >= session.workers.len() {
                    session.selected_worker = Some(session.workers.len() - 1);
                }
            }
            app.confirm_delete = false;
            app.status = "Deleted".to_string();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_delete = false;
            app.status = "Cancelled".to_string();
        }
        _ => {}
    }
}

fn handle_confirm_clear_all(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            let session = app.current_session_mut();
            let count = session.workers.len();
            session.workers.clear();
            session.selected_worker = None;
            session
                .messages
                .push((format!("Cleared {} workers", count), false));
            app.confirm_clear_all = false;
            app.status = "Cleared".to_string();
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_clear_all = false;
            app.status = "Cancelled".to_string();
        }
        _ => {}
    }
}

fn handle_confirm_delete_session(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('y') | KeyCode::Char('Y') => {
            app.delete_current_session();
            app.confirm_delete_session = false;
        }
        KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
            app.confirm_delete_session = false;
            app.status = "Cancelled".to_string();
        }
        _ => {}
    }
}

fn handle_stop_selector(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Up | KeyCode::Char('k') => {
            app.stop_selector_prev();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            app.stop_selector_next();
        }
        KeyCode::Char(' ') => {
            let running_workers = app.get_running_workers();
            if let Some((worker_idx, _)) = running_workers.get(app.stop_selector_cursor) {
                app.toggle_stop_selection(*worker_idx);
            }
        }
        KeyCode::Char('a') | KeyCode::Char('A') => {
            let running_indices: Vec<usize> = app
                .get_running_workers()
                .iter()
                .map(|(idx, _)| *idx)
                .collect();
            if app.stop_selector_selections.len() == running_indices.len() {
                app.stop_selector_selections.clear();
            } else {
                app.stop_selector_selections = running_indices;
            }
        }
        KeyCode::Enter => {
            if !app.stop_selector_selections.is_empty() {
                let mut stopped_ids = Vec::new();
                let mut indices_to_stop: Vec<usize> = app.stop_selector_selections.clone();
                indices_to_stop.sort();
                indices_to_stop.reverse();

                for &idx in &indices_to_stop {
                    if let Some(worker) = app.current_session_mut().workers.get_mut(idx) {
                        stopped_ids.push(worker.id);
                        worker.state = WorkerState::Error;
                        worker.output.push("Interrupted by user".to_string());
                    }
                }

                let count = stopped_ids.len();
                let ids_str: String = stopped_ids
                    .iter()
                    .map(|id| format!("#{}", id))
                    .collect::<Vec<_>>()
                    .join(", ");

                app.current_session_mut()
                    .messages
                    .push((format!("Stopped {} worker(s): {}", count, ids_str), false));
                app.status = format!("Stopped {} worker(s)", count);
                app.reset_stop_selector();
            }
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.reset_stop_selector();
            app.status = "Cancelled".to_string();
        }
        _ => {}
    }
}

fn handle_navigation_mode(app: &mut App, key: KeyEvent) {
    let has_selected_worker = app.current_session().selected_worker.is_some();

    match key.code {
        KeyCode::Char('q') => app.quit = true,
        KeyCode::Char('i') | KeyCode::Enter => app.input_mode = true,
        KeyCode::Char('j') | KeyCode::Down => {
            if app.show_logs {
                app.logs_scroll_state.scroll_by(1);
            } else if has_selected_worker {
                app.current_session_mut().scroll_offset += 1;
            } else {
                app.current_session_mut().select_next_worker();
            }
        }
        KeyCode::Char('k') | KeyCode::Up => {
            if app.show_logs {
                app.logs_scroll_state.scroll_by(-1);
            } else if has_selected_worker {
                app.current_session_mut().scroll_offset =
                    app.current_session().scroll_offset.saturating_sub(1);
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
                app.status = "Delete worker? (y/n)".to_string();
            }
        }
        KeyCode::Char('c') | KeyCode::Char('C') => {
            if !app.current_session().workers.is_empty() {
                app.confirm_clear_all = true;
                app.status = format!(
                    "Clear {} workers? (y/n)",
                    app.current_session().workers.len()
                );
            }
        }
        KeyCode::PageDown | KeyCode::Char('J') => {
            if app.show_logs {
                app.logs_scroll_state.scroll_by(PAGE_SCROLL_LINES as isize);
            } else {
                app.main_scroll_state.scroll_by(LINE_SCROLL_AMOUNT as isize);
            }
        }
        KeyCode::PageUp | KeyCode::Char('K') => {
            if app.show_logs {
                app.logs_scroll_state.scroll_by(-(PAGE_SCROLL_LINES as isize));
            } else {
                app.main_scroll_state.scroll_by(-(LINE_SCROLL_AMOUNT as isize));
            }
        }
        KeyCode::Home | KeyCode::Char('g') => {
            if app.show_logs {
                app.logs_scroll_state.scroll_to_top();
            } else {
                app.main_scroll_state.scroll_to_top();
            }
        }
        KeyCode::End | KeyCode::Char('G') => {
            if app.show_logs {
                app.logs_scroll_state.scroll_to_bottom();
            } else {
                app.main_scroll_state.scroll_to_bottom();
            }
        }
        KeyCode::Char('l') => {
            app.show_logs = !app.show_logs;
            if app.show_logs {
                app.logs_scroll_state.scroll_to_bottom();
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

fn handle_task_plan(
    app: &mut App,
    session_id: usize,
    plan: crate::orchestrator::TaskPlan,
    logs: Vec<String>,
    orch_session_id: String,
) {
    app.orchestrator_logs.extend(logs);
    if let Some(session) = app.find_session_mut(session_id) {
        if session.orchestrator_session_id.is_none() {
            session.orchestrator_session_id = Some(orch_session_id);
        }
        session.workers.clear();
        session.selected_worker = None;
        session
            .messages
            .push((format!("Plan: {}", plan.reasoning), false));
        session
            .messages
            .push((format!("Spawning {} workers...", plan.tasks.len()), false));
        for task in &plan.tasks {
            session
                .messages
                .push((format!("  #{}: {}", task.id, task.description), false));
            session
                .workers
                .push(Worker::new(task.id, task.description.clone()));
        }
    }
    app.status = format!("Running {} workers", plan.tasks.len());
}

fn handle_worker_started(
    app: &mut App,
    session_id: usize,
    worker_id: u32,
    opencode_session_id: String,
) {
    app.orchestrator_logs
        .push(format!("[WORKER] #{} started", worker_id));
    app.update_worker(session_id, worker_id, |worker| {
        worker.session_id = Some(opencode_session_id);
        worker.state = WorkerState::Running;
        worker.output.push("Started...".to_string());
    });
}

fn handle_worker_complete(app: &mut App, session_id: usize, worker_id: u32) {
    app.update_worker(session_id, worker_id, |worker| {
        worker.state = WorkerState::Complete;
        worker.output.push("Complete".to_string());
    });
    if let Some(session) = app.find_session(session_id) {
        if session.workers.iter().all(|w| w.state.is_terminal()) {
            app.status = "All workers complete".to_string();
        }
    }
}

async fn handle_stream_event(
    app: &mut App,
    event: StreamEvent,
    _server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
    match event {
        StreamEvent::Connected => {
            app.orchestrator_logs.push("[SSE] Connected".to_string());
        }
        StreamEvent::PartUpdated { session_id, part } => {
            if let Some(session) = app.find_session_by_worker_session_id(&session_id) {
                if let Some(worker) = session
                    .workers
                    .iter_mut()
                    .find(|w| w.session_id.as_deref() == Some(&session_id))
                {
                    if let Some(text) = &part.text {
                        worker.streaming_content = text.clone();
                        worker.current_tool = None;
                    }
                }
            }
        }
        StreamEvent::ToolCall {
            session_id,
            tool_name,
            status,
            input,
        } => {
            let mut question_to_show: Option<(u32, String, String)> = None;

            if let Some(session) = app.find_session_by_worker_session_id(&session_id) {
                if let Some(worker) = session
                    .workers
                    .iter_mut()
                    .find(|w| w.session_id.as_deref() == Some(&session_id))
                {
                    let display_name = format_tool_display(&tool_name, &input);
                    match status.as_str() {
                        "pending" | "running" => {
                            worker.current_tool = Some(display_name);
                            if tool_name == "question" && status == "running" {
                                let extracted = extract_question_text(&input);
                                if !extracted.is_empty() {
                                    worker.pending_question = Some(extracted.clone());
                                    worker.state = WorkerState::WaitingForInput;
                                    question_to_show =
                                        Some((worker.id, worker.description.clone(), extracted));
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

            if let Some((worker_id, desc, question_text)) = question_to_show {
                app.current_session_mut()
                    .messages
                    .push((format!("Worker #{} ({}) asks:", worker_id, desc), false));
                for line in question_text.lines() {
                    app.current_session_mut()
                        .messages
                        .push((format!("  {}", line), false));
                }
                app.current_session_mut()
                    .messages
                    .push((format!("Reply: /reply #{} <answer>", worker_id), false));
                app.status = format!("Worker #{} waiting for input", worker_id);
            }
        }
        StreamEvent::SessionIdle { session_id } => {
            let mut report_data: Option<(usize, String)> = None;

            if let Some(session) = app.find_session_by_worker_session_id(&session_id) {
                if let Some(worker) = session
                    .workers
                    .iter_mut()
                    .find(|w| w.session_id.as_deref() == Some(&session_id))
                {
                    if worker.pending_question.is_none() {
                        worker.state = WorkerState::Complete;
                        if !worker.streaming_content.is_empty() {
                            worker.output = worker
                                .streaming_content
                                .lines()
                                .map(|s| s.to_string())
                                .collect();
                        }
                        worker.output.push("Complete".to_string());

                        let summary = worker.get_summary();
                        session
                            .messages
                            .push((format!("--- Worker #{} ---", worker.id), false));
                        for line in summary.lines().take(5) {
                            session.messages.push((line.to_string(), false));
                        }
                        worker.streaming_content.clear();
                    }
                }

                let all_done = session.workers.iter().all(|w| w.state.is_terminal());
                if all_done && !session.workers.is_empty() {
                    let summaries: Vec<String> = session
                        .workers
                        .iter()
                        .map(|w| format!("#{}: {}", w.id, w.get_summary()))
                        .collect();
                    report_data = Some((session.id, summaries.join("\n\n")));
                }
            }

            if let Some((ui_session_id, results)) = report_data {
                app.status = "Reporting to orchestrator...".to_string();
                let tx_clone = tx.clone();
                tokio::spawn(async move {
                    let _ = tx_clone
                        .send(AppMessage::ReportToOrchestrator(ui_session_id, results))
                        .await;
                });
            }
        }
        StreamEvent::QuestionAsked {
            session_id,
            request_id,
            questions,
        } => {
            let mut question_info: Option<(u32, String)> = None;

            if let Some(session) = app.find_session_by_worker_session_id(&session_id) {
                if let Some(worker) = session
                    .workers
                    .iter_mut()
                    .find(|w| w.session_id.as_deref() == Some(&session_id))
                {
                    let question_text: String = questions
                        .iter()
                        .map(|q| q.question.clone())
                        .collect::<Vec<_>>()
                        .join("\n");
                    worker.pending_question = Some(question_text.clone());
                    worker.pending_question_request_id = Some(request_id);
                    worker.state = WorkerState::WaitingForInput;
                    question_info = Some((worker.id, question_text));
                }
            }

            if let Some((worker_id, question_text)) = question_info {
                app.current_session_mut()
                    .messages
                    .push((format!("Worker #{} asks:", worker_id), false));
                for line in question_text.lines() {
                    app.current_session_mut()
                        .messages
                        .push((format!("  {}", line), false));
                }
                app.current_session_mut()
                    .messages
                    .push((format!("Reply: /reply #{} <answer>", worker_id), false));
                app.status = format!("Worker #{} waiting", worker_id);
            }
        }
        StreamEvent::PermissionAsked {
            session_id,
            request_id,
            permission,
            patterns,
        } => {
            let mut worker_info: Option<(u32, String)> = None;
            for session in &app.sessions {
                for worker in &session.workers {
                    if worker.session_id.as_deref() == Some(&session_id) {
                        worker_info = Some((worker.id, worker.description.clone()));
                        break;
                    }
                }
            }

            let pending = PendingPermission {
                request_id,
                session_id,
                permission: permission.clone(),
                patterns,
                worker_id: worker_info.as_ref().map(|(id, _)| *id),
                worker_description: worker_info.map(|(_, desc)| desc),
            };
            app.pending_permissions.push(pending);

            if !app.show_permission_dialog {
                app.show_permission_dialog = true;
                app.permission_selector_index = 0;
            }
            app.status = format!("Permission: {}", permission);
        }
        StreamEvent::Error(e) => {
            app.orchestrator_logs.push(format!("[SSE] Error: {}", e));
        }
    }
}

async fn handle_report_to_orchestrator(
    app: &mut App,
    ui_session_id: usize,
    results: String,
    server: &OpenCodeServer,
    tx: &Sender<AppMessage>,
) {
    if let Some(session) = app.find_session(ui_session_id) {
        if let Some(orch_session_id) = &session.orchestrator_session_id {
            let server_clone = server.clone();
            let orch_session_id = orch_session_id.clone();
            let tx_clone = tx.clone();

            tokio::spawn(async move {
                let mut orch = Orchestrator::new(server_clone);
                orch.set_session_id(orch_session_id);

                match orch.report_worker_results(&results).await {
                    Ok(()) => {
                        let _ = tx_clone
                            .send(AppMessage::CommandResult("Results reported".to_string()))
                            .await;
                    }
                    Err(e) => {
                        let _ = tx_clone
                            .send(AppMessage::CommandResult(format!("Report failed: {}", e)))
                            .await;
                    }
                }
            });
        } else {
            app.status = "All workers complete".to_string();
        }
    } else {
        app.status = "All workers complete".to_string();
    }
}

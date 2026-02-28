//! Application state for the TUI.

use crate::orchestrator::Orchestrator;
use crate::server::OpenCodeServer;

use super::commands::{get_suggestions, CommandSuggestion};
use super::messages::{ModelOption, PendingPermission};
use super::session::Session;

/// Main application state.
pub struct App {
    /// Reference to the server client.
    #[allow(dead_code)]
    pub server: OpenCodeServer,
    /// The orchestrator instance.
    #[allow(dead_code)]
    pub orchestrator: Orchestrator,
    /// All sessions.
    pub sessions: Vec<Session>,
    /// Index of the current session.
    pub current_session: usize,
    /// Next session ID to assign.
    pub next_session_id: usize,
    /// Current input text.
    pub input: String,
    /// Cursor position in the input.
    pub cursor_pos: usize,
    /// Whether we're in input mode.
    pub input_mode: bool,
    /// Orchestrator debug logs.
    pub orchestrator_logs: Vec<String>,
    /// Scroll position in logs view.
    pub logs_scroll: usize,
    /// Whether to quit the app.
    pub quit: bool,
    /// Status bar text.
    pub status: String,
    /// Whether to show the logs panel.
    pub show_logs: bool,
    /// Confirmation dialog for worker deletion.
    pub confirm_delete: bool,
    /// Confirmation dialog for clearing all workers.
    pub confirm_clear_all: bool,
    /// Confirmation dialog for session deletion.
    pub confirm_delete_session: bool,
    /// Current autocomplete suggestion index.
    pub autocomplete_index: usize,
    /// Whether to show autocomplete.
    pub show_autocomplete: bool,
    /// Whether to show the model selector.
    pub show_model_selector: bool,
    /// Available model options.
    pub model_options: Vec<ModelOption>,
    /// Selected model index.
    pub model_selector_index: usize,
    /// Pending permission requests.
    pub pending_permissions: Vec<PendingPermission>,
    /// Whether to show the permission dialog.
    pub show_permission_dialog: bool,
    /// Selected permission option index.
    pub permission_selector_index: usize,
}

impl App {
    /// Creates a new App with the given server.
    pub fn new(server: OpenCodeServer) -> Self {
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

    /// Returns a reference to the current session.
    pub fn current_session(&self) -> &Session {
        &self.sessions[self.current_session]
    }

    /// Returns a mutable reference to the current session.
    pub fn current_session_mut(&mut self) -> &mut Session {
        &mut self.sessions[self.current_session]
    }

    /// Creates a new session with an optional name.
    pub fn create_session(&mut self, name: Option<String>) {
        let name = name.unwrap_or_else(|| format!("Session {}", self.next_session_id + 1));
        let session = Session::new(self.next_session_id, name.clone());
        self.next_session_id += 1;
        self.sessions.push(session);
        self.current_session = self.sessions.len() - 1;
        self.status = format!("Created session '{}'", name);
    }

    /// Deletes the current session.
    pub fn delete_current_session(&mut self) {
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

    /// Switches to the next session.
    pub fn next_session(&mut self) {
        if !self.sessions.is_empty() {
            self.current_session = (self.current_session + 1) % self.sessions.len();
        }
    }

    /// Switches to the previous session.
    pub fn prev_session(&mut self) {
        if !self.sessions.is_empty() {
            self.current_session = if self.current_session == 0 {
                self.sessions.len() - 1
            } else {
                self.current_session - 1
            };
        }
    }

    /// Returns command suggestions for the current input.
    pub fn get_current_suggestions(&self) -> Vec<&'static CommandSuggestion> {
        if self.input.starts_with('/') {
            get_suggestions(&self.input)
        } else {
            vec![]
        }
    }

    /// Applies the current autocomplete suggestion.
    pub fn apply_autocomplete(&mut self) {
        let suggestions = self.get_current_suggestions();
        if !suggestions.is_empty() && self.autocomplete_index < suggestions.len() {
            self.input = suggestions[self.autocomplete_index].command.to_string();
            self.cursor_pos = self.input.chars().count();
            self.show_autocomplete = false;
        }
    }

    /// Selects the next autocomplete suggestion.
    pub fn autocomplete_next(&mut self) {
        let suggestions = self.get_current_suggestions();
        if !suggestions.is_empty() {
            self.autocomplete_index = (self.autocomplete_index + 1) % suggestions.len();
        }
    }

    /// Selects the previous autocomplete suggestion.
    pub fn autocomplete_prev(&mut self) {
        let suggestions = self.get_current_suggestions();
        if !suggestions.is_empty() {
            self.autocomplete_index = if self.autocomplete_index == 0 {
                suggestions.len() - 1
            } else {
                self.autocomplete_index - 1
            };
        }
    }

    /// Finds a session that contains a worker with the given OpenCode session ID.
    pub fn find_session_by_worker_session_id(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| {
            s.workers
                .iter()
                .any(|w| w.session_id.as_deref() == Some(session_id))
        })
    }

    /// Adds a log message to the orchestrator logs.
    pub fn log(&mut self, message: String) {
        self.orchestrator_logs.push(message);
    }

    /// Returns true if any dialog is currently showing.
    pub fn has_dialog_open(&self) -> bool {
        self.show_logs
            || self.show_model_selector
            || self.show_permission_dialog
            || self.confirm_delete
            || self.confirm_clear_all
            || self.confirm_delete_session
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_app() -> App {
        let server = OpenCodeServer::new(4096);
        App::new(server)
    }

    #[test]
    fn new_creates_initial_session() {
        let app = create_test_app();
        assert_eq!(app.sessions.len(), 1);
        assert_eq!(app.current_session, 0);
    }

    #[test]
    fn create_session_adds_and_selects() {
        let mut app = create_test_app();
        app.create_session(Some("Test".to_string()));
        assert_eq!(app.sessions.len(), 2);
        assert_eq!(app.current_session, 1);
        assert_eq!(app.current_session().name, "Test");
    }

    #[test]
    fn create_session_generates_name() {
        let mut app = create_test_app();
        app.create_session(None);
        assert!(app.current_session().name.starts_with("Session"));
    }

    #[test]
    fn delete_session_removes_and_adjusts() {
        let mut app = create_test_app();
        app.create_session(Some("Second".to_string()));
        app.create_session(Some("Third".to_string()));
        app.current_session = 2;

        app.delete_current_session();
        assert_eq!(app.sessions.len(), 2);
        assert_eq!(app.current_session, 1);
    }

    #[test]
    fn delete_session_prevents_last() {
        let mut app = create_test_app();
        app.delete_current_session();
        assert_eq!(app.sessions.len(), 1);
        assert!(app.status.contains("Cannot delete"));
    }

    #[test]
    fn next_session_cycles() {
        let mut app = create_test_app();
        app.create_session(None);
        app.create_session(None);
        app.current_session = 0;

        app.next_session();
        assert_eq!(app.current_session, 1);

        app.next_session();
        assert_eq!(app.current_session, 2);

        app.next_session();
        assert_eq!(app.current_session, 0);
    }

    #[test]
    fn prev_session_cycles() {
        let mut app = create_test_app();
        app.create_session(None);
        app.current_session = 0;

        app.prev_session();
        assert_eq!(app.current_session, 1);

        app.prev_session();
        assert_eq!(app.current_session, 0);
    }

    #[test]
    fn autocomplete_cycles() {
        let mut app = create_test_app();
        app.input = "/".to_string();
        let suggestion_count = app.get_current_suggestions().len();

        app.autocomplete_next();
        assert_eq!(app.autocomplete_index, 1 % suggestion_count);

        app.autocomplete_index = 0;
        app.autocomplete_prev();
        assert_eq!(app.autocomplete_index, suggestion_count - 1);
    }

    #[test]
    fn apply_autocomplete_sets_input() {
        let mut app = create_test_app();
        app.input = "/he".to_string();
        app.autocomplete_index = 0;
        app.apply_autocomplete();
        assert_eq!(app.input, "/help");
    }

    #[test]
    fn has_dialog_open_returns_correctly() {
        let mut app = create_test_app();
        assert!(!app.has_dialog_open());

        app.show_logs = true;
        assert!(app.has_dialog_open());

        app.show_logs = false;
        app.show_model_selector = true;
        assert!(app.has_dialog_open());
    }
}

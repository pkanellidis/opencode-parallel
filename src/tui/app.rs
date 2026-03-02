//! Application state for the TUI.

use crate::orchestrator::Orchestrator;
use crate::server::OpenCodeServer;

use super::commands::{get_suggestions, CommandSuggestion};
use super::messages::{ModelOption, PendingPermission};
use super::scroll::{ScrollDirection, ScrollState};
use super::selection::TextSelection;
use super::session::Session;
use super::textarea::EnhancedTextArea;

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
    /// Enhanced text area for multiline input with opencode-style keybindings.
    pub textarea: EnhancedTextArea,
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
    /// Whether to show the stop worker selector.
    pub show_stop_selector: bool,
    /// Selected stop worker indices (for multi-select).
    pub stop_selector_selections: Vec<usize>,
    /// Cursor position in stop selector.
    pub stop_selector_cursor: usize,
    /// Scroll state for main content area (with acceleration).
    pub main_scroll_state: ScrollState,
    /// Scroll state for logs panel (with acceleration).
    pub logs_scroll_state: ScrollState,
    /// Current text selection (for copy).
    pub selection: Option<TextSelection>,
    /// Content lines for the main view (used for selection/copy).
    pub content_lines: Vec<String>,
    /// Currently selected model (provider/model).
    pub current_model: Option<String>,
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
            textarea: EnhancedTextArea::new(),
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
            show_stop_selector: false,
            stop_selector_selections: Vec::new(),
            stop_selector_cursor: 0,
            main_scroll_state: ScrollState::new(),
            logs_scroll_state: ScrollState::new(),
            selection: None,
            content_lines: Vec::new(),
            current_model: None,
        }
    }

    /// Handle scroll event for the appropriate panel.
    pub fn handle_scroll(&mut self, direction: ScrollDirection) {
        if self.show_logs {
            self.logs_scroll_state.handle_scroll(direction);
        } else {
            self.main_scroll_state.handle_scroll(direction);
        }
    }

    /// Update scroll state (call each frame for momentum scrolling).
    pub fn tick_scroll(&mut self) -> bool {
        let main_changed = self.main_scroll_state.tick();
        let logs_changed = self.logs_scroll_state.tick();
        main_changed || logs_changed
    }

    /// Get main scroll offset.
    pub fn main_scroll(&self) -> usize {
        self.main_scroll_state.offset
    }

    /// Get logs scroll offset.
    pub fn logs_scroll(&self) -> usize {
        self.logs_scroll_state.offset
    }

    /// Set main scroll dimensions.
    pub fn set_main_scroll_dimensions(&mut self, total: usize, visible: usize) {
        self.main_scroll_state.set_dimensions(total, visible);
    }

    /// Set logs scroll dimensions.
    pub fn set_logs_scroll_dimensions(&mut self, total: usize, visible: usize) {
        self.logs_scroll_state.set_dimensions(total, visible);
    }

    /// Returns the current input text (all lines joined).
    pub fn input(&self) -> String {
        self.textarea.input()
    }

    /// Clears the input textarea.
    pub fn clear_input(&mut self) {
        self.textarea.clear();
    }

    /// Returns whether the input is empty.
    pub fn input_is_empty(&self) -> bool {
        self.textarea.is_empty()
    }

    /// Returns whether the input starts with a given prefix.
    pub fn input_starts_with(&self, prefix: &str) -> bool {
        self.textarea.starts_with(prefix)
    }

    /// Sets the input text.
    pub fn set_input(&mut self, text: &str) {
        self.textarea.set_input(text);
    }

    /// Adds the current input to history before clearing.
    pub fn submit_input(&mut self) -> String {
        let input = self.input();
        self.textarea.add_to_history(input.clone());
        self.clear_input();
        input
    }

    /// Sets the current model display string.
    pub fn set_current_model(&mut self, model: Option<String>) {
        self.current_model = model;
    }

    /// Clears the current text selection.
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Starts a new text selection at the given position.
    pub fn start_selection(&mut self, col: u16, row: u16) {
        self.selection = Some(TextSelection::new(col, row));
    }

    /// Updates the current selection end position.
    pub fn update_selection(&mut self, col: u16, row: u16) {
        if let Some(ref mut sel) = self.selection {
            sel.update(col, row);
        }
    }

    /// Finishes the current selection.
    pub fn finish_selection(&mut self) {
        if let Some(ref mut sel) = self.selection {
            sel.finish();
            if sel.is_empty() {
                self.selection = None;
            }
        }
    }

    /// Gets the selected text from content_lines.
    pub fn get_selected_text(&self) -> Option<String> {
        let sel = self.selection.as_ref()?;
        if sel.is_empty() {
            return None;
        }

        let (start, end) = sel.normalized();
        let mut lines = Vec::new();

        for row in start.row..=end.row {
            if let Some(line) = self.content_lines.get(row as usize) {
                let line_len = line.len() as u16;
                let col_start = if row == start.row { start.col } else { 0 };
                let col_end = if row == end.row {
                    end.col.min(line_len)
                } else {
                    line_len
                };

                if col_start < col_end && (col_start as usize) < line.len() {
                    let end_idx = (col_end as usize).min(line.len());
                    lines.push(line[col_start as usize..end_idx].to_string());
                }
            }
        }

        if lines.is_empty() {
            None
        } else {
            Some(lines.join("\n"))
        }
    }

    /// Copies selected text to clipboard.
    pub fn copy_selection(&mut self) -> Option<String> {
        let text = self.get_selected_text();
        if text.is_some() {
            self.clear_selection();
        }
        text
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
        let input = self.input();
        if input.starts_with('/') {
            get_suggestions(&input)
        } else {
            vec![]
        }
    }

    /// Applies the current autocomplete suggestion.
    pub fn apply_autocomplete(&mut self) {
        let suggestions = self.get_current_suggestions();
        if !suggestions.is_empty() && self.autocomplete_index < suggestions.len() {
            let command = suggestions[self.autocomplete_index].command.to_string();
            self.set_input(&command);
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

    /// Finds a mutable session by its ID.
    pub fn find_session_mut(&mut self, session_id: usize) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|s| s.id == session_id)
    }

    /// Finds a session by its ID (immutable).
    pub fn find_session(&self, session_id: usize) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == session_id)
    }

    /// Updates a worker in a session by applying a closure.
    /// Returns true if the worker was found and updated.
    pub fn update_worker<F>(&mut self, session_id: usize, worker_id: u32, f: F) -> bool
    where
        F: FnOnce(&mut super::worker::Worker),
    {
        if let Some(session) = self.find_session_mut(session_id) {
            if let Some(worker) = session.workers.iter_mut().find(|w| w.id == worker_id) {
                f(worker);
                return true;
            }
        }
        false
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
            || self.show_stop_selector
            || self.confirm_delete
            || self.confirm_clear_all
            || self.confirm_delete_session
    }

    /// Returns running (non-terminal) workers in the current session.
    pub fn get_running_workers(&self) -> Vec<(usize, &super::worker::Worker)> {
        self.current_session()
            .workers
            .iter()
            .enumerate()
            .filter(|(_, w)| !w.state.is_terminal())
            .collect()
    }

    /// Toggles a worker selection in the stop selector.
    pub fn toggle_stop_selection(&mut self, index: usize) {
        if self.stop_selector_selections.contains(&index) {
            self.stop_selector_selections.retain(|&i| i != index);
        } else {
            self.stop_selector_selections.push(index);
        }
    }

    /// Moves the stop selector cursor down.
    pub fn stop_selector_next(&mut self) {
        let running_count = self.get_running_workers().len();
        if running_count > 0 {
            self.stop_selector_cursor = (self.stop_selector_cursor + 1) % running_count;
        }
    }

    /// Moves the stop selector cursor up.
    pub fn stop_selector_prev(&mut self) {
        let running_count = self.get_running_workers().len();
        if running_count > 0 {
            self.stop_selector_cursor = if self.stop_selector_cursor == 0 {
                running_count - 1
            } else {
                self.stop_selector_cursor - 1
            };
        }
    }

    /// Resets the stop selector state.
    pub fn reset_stop_selector(&mut self) {
        self.show_stop_selector = false;
        self.stop_selector_selections.clear();
        self.stop_selector_cursor = 0;
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
        app.set_input("/");
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
        app.set_input("/he");
        app.autocomplete_index = 0;
        app.apply_autocomplete();
        assert_eq!(app.input(), "/help");
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

    #[test]
    fn show_logs_toggle() {
        let mut app = create_test_app();
        assert!(!app.show_logs);

        app.show_logs = true;
        assert!(app.show_logs);
        assert!(app.has_dialog_open());

        app.show_logs = false;
        assert!(!app.show_logs);
    }

    #[test]
    fn orchestrator_logs_can_be_added() {
        let mut app = create_test_app();
        assert!(app.orchestrator_logs.is_empty());

        app.log("Test log message".to_string());
        assert_eq!(app.orchestrator_logs.len(), 1);
        assert_eq!(app.orchestrator_logs[0], "Test log message");

        app.log("Another log".to_string());
        assert_eq!(app.orchestrator_logs.len(), 2);
    }

    #[test]
    fn logs_scroll_defaults_to_zero() {
        let app = create_test_app();
        assert_eq!(app.logs_scroll(), 0);
    }

    #[test]
    fn logs_scroll_can_be_modified() {
        let mut app = create_test_app();
        app.logs_scroll_state.set_dimensions(100, 20);

        app.logs_scroll_state.scroll_to(10);
        assert_eq!(app.logs_scroll(), 10);

        app.logs_scroll_state.scroll_by(-5);
        assert_eq!(app.logs_scroll(), 5);

        app.logs_scroll_state.scroll_by(3);
        assert_eq!(app.logs_scroll(), 8);
    }
}

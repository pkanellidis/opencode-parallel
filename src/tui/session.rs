//! Session management for the TUI.
//!
//! Sessions group related workers and conversations together.

use super::worker::Worker;

/// A TUI session containing workers and chat messages.
pub struct Session {
    /// Unique session ID within the app.
    pub id: usize,
    /// Display name for the session.
    pub name: String,
    /// Chat messages: (text, is_user_message).
    pub messages: Vec<(String, bool)>,
    /// Workers in this session.
    pub workers: Vec<Worker>,
    /// Currently selected worker index.
    pub selected_worker: Option<usize>,
    /// Scroll offset for the message view.
    pub scroll_offset: usize,
    /// OpenCode orchestrator session ID.
    pub orchestrator_session_id: Option<String>,
}

impl Session {
    /// Creates a new session with the given ID and name.
    pub fn new(id: usize, name: String) -> Self {
        let welcome_msg = format!("Session '{}' created.", &name);
        Self {
            id,
            name,
            messages: vec![
                (welcome_msg, false),
                ("Type a task and press Enter to start.".to_string(), false),
                (String::new(), false),
            ],
            workers: Vec::new(),
            selected_worker: None,
            scroll_offset: 0,
            orchestrator_session_id: None,
        }
    }

    /// Adds a message to the session.
    pub fn add_message(&mut self, text: String, is_user: bool) {
        self.messages.push((text, is_user));
    }

    /// Adds a system message (not from user).
    pub fn add_system_message(&mut self, text: String) {
        self.add_message(text, false);
    }

    /// Clears all messages except the welcome message.
    pub fn clear_messages(&mut self) {
        let welcome = format!("Session '{}' - messages cleared.", &self.name);
        self.messages = vec![(welcome, false), (String::new(), false)];
    }

    /// Selects the next worker in the list.
    pub fn select_next_worker(&mut self) {
        if self.workers.is_empty() {
            return;
        }
        self.selected_worker = Some(match self.selected_worker {
            Some(i) if i < self.workers.len() - 1 => i + 1,
            _ => 0,
        });
    }

    /// Selects the previous worker in the list.
    pub fn select_prev_worker(&mut self) {
        if self.workers.is_empty() {
            return;
        }
        self.selected_worker = Some(match self.selected_worker {
            Some(i) if i > 0 => i - 1,
            _ => self.workers.len() - 1,
        });
    }

    /// Returns the currently selected worker, if any.
    pub fn selected_worker(&self) -> Option<&Worker> {
        self.selected_worker.and_then(|i| self.workers.get(i))
    }

    /// Returns a mutable reference to the currently selected worker.
    pub fn selected_worker_mut(&mut self) -> Option<&mut Worker> {
        self.selected_worker.and_then(|i| self.workers.get_mut(i))
    }

    /// Finds a worker by its ID.
    pub fn find_worker(&self, id: u32) -> Option<&Worker> {
        self.workers.iter().find(|w| w.id == id)
    }

    /// Finds a mutable reference to a worker by its ID.
    pub fn find_worker_mut(&mut self, id: u32) -> Option<&mut Worker> {
        self.workers.iter_mut().find(|w| w.id == id)
    }

    /// Finds a worker by its OpenCode session ID.
    pub fn find_worker_by_session(&self, session_id: &str) -> Option<&Worker> {
        self.workers
            .iter()
            .find(|w| w.session_id.as_deref() == Some(session_id))
    }

    /// Finds a mutable reference to a worker by its OpenCode session ID.
    pub fn find_worker_by_session_mut(&mut self, session_id: &str) -> Option<&mut Worker> {
        self.workers
            .iter_mut()
            .find(|w| w.session_id.as_deref() == Some(session_id))
    }

    /// Returns the number of completed workers.
    pub fn completed_worker_count(&self) -> usize {
        self.workers
            .iter()
            .filter(|w| w.state.is_terminal())
            .count()
    }

    /// Returns true if all workers are in a terminal state.
    pub fn all_workers_done(&self) -> bool {
        !self.workers.is_empty() && self.workers.iter().all(|w| w.state.is_terminal())
    }

    /// Removes a worker by index.
    pub fn remove_worker(&mut self, index: usize) -> Option<Worker> {
        if index < self.workers.len() {
            let worker = self.workers.remove(index);
            // Adjust selection
            if self.workers.is_empty() {
                self.selected_worker = None;
            } else if let Some(sel) = self.selected_worker {
                if sel >= self.workers.len() {
                    self.selected_worker = Some(self.workers.len() - 1);
                }
            }
            Some(worker)
        } else {
            None
        }
    }

    /// Clears all workers from the session.
    pub fn clear_workers(&mut self) {
        self.workers.clear();
        self.selected_worker = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::worker::WorkerState;

    #[test]
    fn new_creates_session_with_welcome_message() {
        let session = Session::new(1, "Test".to_string());
        assert_eq!(session.id, 1);
        assert_eq!(session.name, "Test");
        assert!(!session.messages.is_empty());
        assert!(session.messages[0].0.contains("Test"));
    }

    #[test]
    fn add_message_appends_to_messages() {
        let mut session = Session::new(1, "Test".to_string());
        let initial_count = session.messages.len();
        session.add_message("Hello".to_string(), true);
        assert_eq!(session.messages.len(), initial_count + 1);
        assert_eq!(session.messages.last().unwrap().0, "Hello");
        assert!(session.messages.last().unwrap().1);
    }

    #[test]
    fn clear_messages_keeps_welcome() {
        let mut session = Session::new(1, "Test".to_string());
        session.add_message("One".to_string(), true);
        session.add_message("Two".to_string(), false);
        session.clear_messages();
        assert!(session.messages.len() <= 2);
        assert!(session.messages[0].0.contains("cleared"));
    }

    #[test]
    fn select_next_worker_cycles() {
        let mut session = Session::new(1, "Test".to_string());
        session.workers.push(Worker::new(1, "W1".to_string()));
        session.workers.push(Worker::new(2, "W2".to_string()));

        session.select_next_worker();
        assert_eq!(session.selected_worker, Some(0));

        session.select_next_worker();
        assert_eq!(session.selected_worker, Some(1));

        session.select_next_worker();
        assert_eq!(session.selected_worker, Some(0)); // Wraps
    }

    #[test]
    fn select_prev_worker_cycles() {
        let mut session = Session::new(1, "Test".to_string());
        session.workers.push(Worker::new(1, "W1".to_string()));
        session.workers.push(Worker::new(2, "W2".to_string()));

        session.select_prev_worker();
        assert_eq!(session.selected_worker, Some(1)); // Wraps to end

        session.select_prev_worker();
        assert_eq!(session.selected_worker, Some(0));
    }

    #[test]
    fn find_worker_returns_correct_worker() {
        let mut session = Session::new(1, "Test".to_string());
        session.workers.push(Worker::new(1, "W1".to_string()));
        session.workers.push(Worker::new(2, "W2".to_string()));

        let worker = session.find_worker(2);
        assert!(worker.is_some());
        assert_eq!(worker.unwrap().id, 2);
    }

    #[test]
    fn find_worker_by_session_works() {
        let mut session = Session::new(1, "Test".to_string());
        let mut worker = Worker::new(1, "W1".to_string());
        worker.session_id = Some("ses_123".to_string());
        session.workers.push(worker);

        let found = session.find_worker_by_session("ses_123");
        assert!(found.is_some());
        assert_eq!(found.unwrap().id, 1);
    }

    #[test]
    fn completed_worker_count_counts_terminal_states() {
        let mut session = Session::new(1, "Test".to_string());

        let mut w1 = Worker::new(1, "W1".to_string());
        w1.state = WorkerState::Complete;
        session.workers.push(w1);

        let mut w2 = Worker::new(2, "W2".to_string());
        w2.state = WorkerState::Running;
        session.workers.push(w2);

        let mut w3 = Worker::new(3, "W3".to_string());
        w3.state = WorkerState::Error;
        session.workers.push(w3);

        assert_eq!(session.completed_worker_count(), 2);
    }

    #[test]
    fn all_workers_done_returns_correctly() {
        let mut session = Session::new(1, "Test".to_string());

        assert!(!session.all_workers_done()); // No workers

        let mut w1 = Worker::new(1, "W1".to_string());
        w1.state = WorkerState::Complete;
        session.workers.push(w1);

        assert!(session.all_workers_done());

        session.workers.push(Worker::new(2, "W2".to_string())); // Starting state

        assert!(!session.all_workers_done());
    }

    #[test]
    fn remove_worker_adjusts_selection() {
        let mut session = Session::new(1, "Test".to_string());
        session.workers.push(Worker::new(1, "W1".to_string()));
        session.workers.push(Worker::new(2, "W2".to_string()));
        session.selected_worker = Some(1);

        session.remove_worker(1);
        assert_eq!(session.selected_worker, Some(0));
    }
}

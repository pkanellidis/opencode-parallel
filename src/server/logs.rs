//! Thread-safe logging for server operations.

use std::sync::{Arc, Mutex};

/// Thread-safe log collector for server operations.
///
/// Logs are stored in memory and can be retrieved for display in the UI.
#[derive(Clone, Default, Debug)]
pub struct ServerLogs {
    logs: Arc<Mutex<Vec<String>>>,
}

impl ServerLogs {
    /// Creates a new empty log collector.
    pub fn new() -> Self {
        Self {
            logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Adds a timestamped log message.
    pub fn log(&self, message: impl Into<String>) {
        let timestamp = chrono::Local::now().format("%H:%M:%S%.3f");
        if let Ok(mut logs) = self.logs.lock() {
            logs.push(format!("[{}] {}", timestamp, message.into()));
        }
    }

    /// Returns all collected log messages.
    pub fn get_logs(&self) -> Vec<String> {
        self.logs.lock().map(|l| l.clone()).unwrap_or_default()
    }

    /// Clears all log messages.
    pub fn clear(&self) {
        if let Ok(mut logs) = self.logs.lock() {
            logs.clear();
        }
    }

    /// Returns the number of log entries.
    pub fn len(&self) -> usize {
        self.logs.lock().map(|l| l.len()).unwrap_or(0)
    }

    /// Returns true if there are no log entries.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_empty_logs() {
        let logs = ServerLogs::new();
        assert!(logs.is_empty());
        assert_eq!(logs.len(), 0);
    }

    #[test]
    fn log_adds_entry() {
        let logs = ServerLogs::new();
        logs.log("test message");
        assert_eq!(logs.len(), 1);
        assert!(!logs.is_empty());
    }

    #[test]
    fn log_entry_contains_timestamp_and_message() {
        let logs = ServerLogs::new();
        logs.log("hello world");
        let entries = logs.get_logs();
        assert!(entries[0].contains("hello world"));
        assert!(entries[0].starts_with('['));
    }

    #[test]
    fn clear_removes_all_entries() {
        let logs = ServerLogs::new();
        logs.log("one");
        logs.log("two");
        assert_eq!(logs.len(), 2);
        logs.clear();
        assert!(logs.is_empty());
    }

    #[test]
    fn clone_shares_state() {
        let logs1 = ServerLogs::new();
        let logs2 = logs1.clone();
        logs1.log("from logs1");
        assert_eq!(logs2.len(), 1);
    }
}

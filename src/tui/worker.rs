//! Worker state and management.
//!
//! Workers are parallel task executors that run in separate OpenCode sessions.

use crate::utils::truncate_str;

/// The current state of a worker.
#[derive(Clone, PartialEq, Debug)]
pub enum WorkerState {
    /// Worker is initializing.
    Starting,
    /// Worker is actively processing.
    Running,
    /// Worker is waiting for user input (e.g., answering a question).
    WaitingForInput,
    /// Worker has finished successfully.
    Complete,
    /// Worker encountered an error.
    Error,
}

impl WorkerState {
    /// Returns a symbol representing the worker state.
    pub fn symbol(&self) -> &'static str {
        match self {
            WorkerState::Starting => "◐",
            WorkerState::Running => "◐",
            WorkerState::WaitingForInput => "❓",
            WorkerState::Complete => "✓",
            WorkerState::Error => "✗",
        }
    }

    /// Returns true if the worker is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(self, WorkerState::Complete | WorkerState::Error)
    }
}

/// A parallel worker executing a task.
pub struct Worker {
    /// Unique worker ID within the session.
    pub id: u32,
    /// Brief description of the task.
    pub description: String,
    /// OpenCode session ID for this worker.
    pub session_id: Option<String>,
    /// Current state.
    pub state: WorkerState,
    /// Final output lines.
    pub output: Vec<String>,
    /// Streaming content (updated in real-time).
    pub streaming_content: String,
    /// Currently executing tool, if any.
    pub current_tool: Option<String>,
    /// History of completed tool calls.
    pub tool_history: Vec<String>,
    /// Pending question text, if waiting for input.
    pub pending_question: Option<String>,
    /// Request ID for the pending question.
    pub pending_question_request_id: Option<String>,
}

impl Worker {
    /// Creates a new worker with the given ID and description.
    pub fn new(id: u32, description: String) -> Self {
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

    /// Returns lines to display for this worker's output.
    pub fn get_display_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();

        // Show completed tools
        for tool in &self.tool_history {
            lines.push(format!("✓ {}", tool));
        }

        // Show current tool
        if let Some(tool) = &self.current_tool {
            lines.push(format!("⚙ {}...", tool));
        }

        // Add separator if we have tool history
        if !self.tool_history.is_empty() || self.current_tool.is_some() {
            lines.push(String::new());
        }

        // Show streaming content or final output
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

    /// Returns a summary of the worker's output for reporting.
    pub fn get_summary(&self) -> String {
        let content = if !self.streaming_content.is_empty() {
            &self.streaming_content
        } else {
            &self.output.join("\n")
        };

        let lines: Vec<&str> = content
            .lines()
            .filter(|l| !l.trim().is_empty())
            .filter(|l| !l.starts_with('✓') && !l.starts_with('⚙') && !l.starts_with('⏳'))
            .collect();

        if lines.is_empty() {
            return format!("Worker #{} completed (no output)", self.id);
        }

        let summary: String = lines
            .iter()
            .take(10)
            .copied()
            .collect::<Vec<&str>>()
            .join("\n");

        if lines.len() > 10 {
            format!("{}...", summary)
        } else {
            summary
        }
    }

    /// Returns a short display string for the worker list.
    pub fn list_display(&self) -> String {
        format!(
            "{} #{} {}",
            self.state.symbol(),
            self.id,
            truncate_str(&self.description, 20)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod worker_state_tests {
        use super::*;

        #[test]
        fn symbol_returns_correct_values() {
            assert_eq!(WorkerState::Starting.symbol(), "◐");
            assert_eq!(WorkerState::Complete.symbol(), "✓");
            assert_eq!(WorkerState::Error.symbol(), "✗");
            assert_eq!(WorkerState::WaitingForInput.symbol(), "❓");
        }

        #[test]
        fn is_terminal_returns_true_for_complete() {
            assert!(WorkerState::Complete.is_terminal());
        }

        #[test]
        fn is_terminal_returns_true_for_error() {
            assert!(WorkerState::Error.is_terminal());
        }

        #[test]
        fn is_terminal_returns_false_for_running() {
            assert!(!WorkerState::Running.is_terminal());
        }

        #[test]
        fn is_terminal_returns_false_for_waiting() {
            assert!(!WorkerState::WaitingForInput.is_terminal());
        }
    }

    mod worker_tests {
        use super::*;

        #[test]
        fn new_creates_worker_in_starting_state() {
            let worker = Worker::new(1, "Test task".to_string());
            assert_eq!(worker.id, 1);
            assert_eq!(worker.description, "Test task");
            assert_eq!(worker.state, WorkerState::Starting);
            assert!(worker.session_id.is_none());
        }

        #[test]
        fn get_display_lines_shows_tool_history() {
            let mut worker = Worker::new(1, "Test".to_string());
            worker.tool_history.push("read file.txt".to_string());
            worker.tool_history.push("write output.txt".to_string());

            let lines = worker.get_display_lines();
            assert!(lines[0].contains("✓ read file.txt"));
            assert!(lines[1].contains("✓ write output.txt"));
        }

        #[test]
        fn get_display_lines_shows_current_tool() {
            let mut worker = Worker::new(1, "Test".to_string());
            worker.current_tool = Some("bash npm install".to_string());

            let lines = worker.get_display_lines();
            assert!(lines[0].contains("⚙ bash npm install..."));
        }

        #[test]
        fn get_display_lines_shows_streaming_content() {
            let mut worker = Worker::new(1, "Test".to_string());
            worker.streaming_content = "Line 1\nLine 2".to_string();

            let lines = worker.get_display_lines();
            assert!(lines.contains(&"Line 1".to_string()));
            assert!(lines.contains(&"Line 2".to_string()));
        }

        #[test]
        fn get_summary_returns_content_summary() {
            let mut worker = Worker::new(1, "Test".to_string());
            worker.streaming_content = "This is the result\nWith multiple lines".to_string();

            let summary = worker.get_summary();
            assert!(summary.contains("This is the result"));
        }

        #[test]
        fn get_summary_returns_default_for_empty_output() {
            let worker = Worker::new(1, "Test".to_string());
            let summary = worker.get_summary();
            assert!(summary.contains("Worker #1 completed"));
        }

        #[test]
        fn get_summary_truncates_long_output() {
            let mut worker = Worker::new(1, "Test".to_string());
            worker.streaming_content = (0..20)
                .map(|i| format!("Line {}", i))
                .collect::<Vec<_>>()
                .join("\n");

            let summary = worker.get_summary();
            assert!(summary.ends_with("..."));
        }

        #[test]
        fn list_display_shows_state_and_description() {
            let worker = Worker::new(1, "Build the frontend".to_string());
            let display = worker.list_display();
            assert!(display.contains("◐"));
            assert!(display.contains("#1"));
            assert!(display.contains("Build the frontend"));
        }
    }
}

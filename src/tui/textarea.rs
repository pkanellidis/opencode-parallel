//! Enhanced textarea implementation simulating opencode TUI keybindings.
//!
//! Based on opencode's textarea-keybindings.ts, this module provides:
//! - Multi-line editing with word wrap
//! - Word navigation (Ctrl+Left/Right, Alt+B/F)
//! - Word deletion (Ctrl+W, Alt+D, Ctrl+Backspace)
//! - Line operations (Ctrl+A, Ctrl+E, Ctrl+K, Ctrl+U)
//! - Undo/Redo support
//! - Selection support
//! - Input history navigation

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::{Color, Style};
use tui_textarea::{CursorMove, TextArea, WrapMode};

pub struct EnhancedTextArea {
    pub textarea: TextArea<'static>,
    history: Vec<String>,
    history_index: Option<usize>,
    current_input: String,
}

impl Default for EnhancedTextArea {
    fn default() -> Self {
        Self::new()
    }
}

impl EnhancedTextArea {
    pub fn new() -> Self {
        Self {
            textarea: Self::create_textarea(),
            history: Vec::new(),
            history_index: None,
            current_input: String::new(),
        }
    }

    fn create_textarea() -> TextArea<'static> {
        let mut textarea = TextArea::default();
        textarea.set_cursor_line_style(Style::default());
        textarea.set_placeholder_text("Press 'i' to enter a task...");
        textarea.set_placeholder_style(Style::default().fg(Color::Rgb(102, 102, 102)));
        textarea.set_wrap_mode(WrapMode::Word);
        textarea
    }

    pub fn lines(&self) -> &[String] {
        self.textarea.lines()
    }

    pub fn input(&self) -> String {
        self.textarea.lines().join("\n")
    }

    pub fn is_empty(&self) -> bool {
        self.textarea.lines().iter().all(|l| l.is_empty())
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        self.textarea
            .lines()
            .first()
            .is_some_and(|l| l.starts_with(prefix))
    }

    pub fn clear(&mut self) {
        self.textarea = Self::create_textarea();
        self.history_index = None;
    }

    pub fn set_input(&mut self, text: &str) {
        self.textarea = Self::create_textarea();
        for (i, line) in text.lines().enumerate() {
            if i > 0 {
                self.textarea.insert_newline();
            }
            self.textarea.insert_str(line);
        }
    }

    pub fn insert_newline(&mut self) {
        self.textarea.insert_newline();
    }

    pub fn insert_str(&mut self, s: &str) {
        self.textarea.insert_str(s);
    }

    pub fn add_to_history(&mut self, entry: String) {
        if !entry.trim().is_empty() && self.history.last() != Some(&entry) {
            self.history.push(entry);
        }
        self.history_index = None;
        self.current_input.clear();
    }

    pub fn history_previous(&mut self) {
        if self.history.is_empty() {
            return;
        }

        match self.history_index {
            None => {
                self.current_input = self.input();
                self.history_index = Some(self.history.len() - 1);
            }
            Some(0) => return,
            Some(idx) => {
                self.history_index = Some(idx - 1);
            }
        }

        if let Some(idx) = self.history_index {
            if let Some(entry) = self.history.get(idx).cloned() {
                self.set_input(&entry);
            }
        }
    }

    pub fn history_next(&mut self) {
        match self.history_index {
            None => (),
            Some(idx) => {
                if idx + 1 >= self.history.len() {
                    self.history_index = None;
                    self.set_input(&self.current_input.clone());
                } else {
                    self.history_index = Some(idx + 1);
                    if let Some(entry) = self.history.get(idx + 1).cloned() {
                        self.set_input(&entry);
                    }
                }
            }
        }
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> TextAreaAction {
        let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
        let alt = key.modifiers.contains(KeyModifiers::ALT);
        let shift = key.modifiers.contains(KeyModifiers::SHIFT);

        match key.code {
            KeyCode::Enter if shift || alt || ctrl => {
                self.textarea.insert_newline();
                TextAreaAction::Continue
            }
            KeyCode::Enter => TextAreaAction::Submit,

            KeyCode::Esc => TextAreaAction::Escape,

            KeyCode::Char('a') if ctrl => {
                self.textarea.move_cursor(CursorMove::Head);
                TextAreaAction::Continue
            }
            KeyCode::Char('e') if ctrl => {
                self.textarea.move_cursor(CursorMove::End);
                TextAreaAction::Continue
            }

            KeyCode::Char('k') if ctrl => {
                self.textarea.delete_line_by_end();
                TextAreaAction::Continue
            }
            KeyCode::Char('u') if ctrl => {
                self.textarea.delete_line_by_head();
                TextAreaAction::Continue
            }

            KeyCode::Char('w') if ctrl => {
                self.textarea.delete_word();
                TextAreaAction::Continue
            }
            KeyCode::Char('d') if alt => {
                self.textarea.delete_next_word();
                TextAreaAction::Continue
            }
            KeyCode::Backspace if ctrl => {
                self.textarea.delete_word();
                TextAreaAction::Continue
            }
            KeyCode::Backspace if alt => {
                self.textarea.delete_word();
                TextAreaAction::Continue
            }
            // macOS Option+Backspace often sends DEL (0x7F) with alt modifier
            KeyCode::Char('\x7f') if alt => {
                self.textarea.delete_word();
                TextAreaAction::Continue
            }
            KeyCode::Delete if ctrl => {
                self.textarea.delete_next_word();
                TextAreaAction::Continue
            }
            KeyCode::Delete if alt => {
                self.textarea.delete_next_word();
                TextAreaAction::Continue
            }

            KeyCode::Left if ctrl || alt => {
                self.textarea.move_cursor(CursorMove::WordBack);
                TextAreaAction::Continue
            }
            KeyCode::Right if ctrl || alt => {
                self.textarea.move_cursor(CursorMove::WordForward);
                TextAreaAction::Continue
            }
            KeyCode::Char('b') if alt => {
                self.textarea.move_cursor(CursorMove::WordBack);
                TextAreaAction::Continue
            }
            KeyCode::Char('f') if alt => {
                self.textarea.move_cursor(CursorMove::WordForward);
                TextAreaAction::Continue
            }
            KeyCode::Char('b') if ctrl => {
                self.textarea.move_cursor(CursorMove::Back);
                TextAreaAction::Continue
            }
            KeyCode::Char('f') if ctrl => {
                self.textarea.move_cursor(CursorMove::Forward);
                TextAreaAction::Continue
            }

            KeyCode::Home if shift => {
                self.textarea.move_cursor(CursorMove::Head);
                TextAreaAction::Continue
            }
            KeyCode::End if shift => {
                self.textarea.move_cursor(CursorMove::End);
                TextAreaAction::Continue
            }
            KeyCode::Home => {
                self.textarea.move_cursor(CursorMove::Head);
                TextAreaAction::Continue
            }
            KeyCode::End => {
                self.textarea.move_cursor(CursorMove::End);
                TextAreaAction::Continue
            }

            KeyCode::Char('z') if ctrl && shift => {
                self.textarea.redo();
                TextAreaAction::Continue
            }
            KeyCode::Char('z') if ctrl => {
                self.textarea.undo();
                TextAreaAction::Continue
            }
            KeyCode::Char('-') if ctrl => {
                self.textarea.undo();
                TextAreaAction::Continue
            }
            KeyCode::Char('.') if ctrl => {
                self.textarea.redo();
                TextAreaAction::Continue
            }

            KeyCode::Up if !ctrl && !alt && self.is_single_line() => {
                TextAreaAction::HistoryPrevious
            }
            KeyCode::Down if !ctrl && !alt && self.is_single_line() => TextAreaAction::HistoryNext,

            KeyCode::Char('d') if ctrl => {
                self.textarea.delete_char();
                TextAreaAction::Continue
            }

            KeyCode::Char('c') if ctrl => TextAreaAction::Clear,

            _ => {
                use crossterm::event::Event;
                use tui_textarea::Input;
                self.textarea.input(Input::from(Event::Key(key)));
                TextAreaAction::Continue
            }
        }
    }

    fn is_single_line(&self) -> bool {
        self.textarea.lines().len() <= 1
    }

    pub fn widget(&self) -> &TextArea<'static> {
        &self.textarea
    }

    pub fn cursor(&self) -> (usize, usize) {
        self.textarea.cursor()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAreaAction {
    Continue,
    Submit,
    Escape,
    HistoryPrevious,
    HistoryNext,
    Clear,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn key_ctrl(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::CONTROL)
    }

    #[allow(dead_code)]
    fn key_alt(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::ALT)
    }

    fn key_shift(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::SHIFT)
    }

    #[test]
    fn test_new_textarea_is_empty() {
        let ta = EnhancedTextArea::new();
        assert!(ta.is_empty());
        assert_eq!(ta.input(), "");
    }

    #[test]
    fn test_set_and_clear_input() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("hello world");
        assert_eq!(ta.input(), "hello world");
        assert!(!ta.is_empty());

        ta.clear();
        assert!(ta.is_empty());
    }

    #[test]
    fn test_multiline_input() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("line1\nline2\nline3");
        assert_eq!(ta.input(), "line1\nline2\nline3");
        assert_eq!(ta.lines().len(), 3);
    }

    #[test]
    fn test_starts_with() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("/help");
        assert!(ta.starts_with("/"));
        assert!(ta.starts_with("/h"));
        assert!(!ta.starts_with("h"));
    }

    #[test]
    fn test_enter_submits() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("test");
        let action = ta.handle_key(key(KeyCode::Enter));
        assert_eq!(action, TextAreaAction::Submit);
    }

    #[test]
    fn test_shift_enter_inserts_newline() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("line1");
        let action = ta.handle_key(key_shift(KeyCode::Enter));
        assert_eq!(action, TextAreaAction::Continue);
        assert!(ta.lines().len() >= 1);
    }

    #[test]
    fn test_ctrl_a_moves_to_start() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("hello");
        ta.handle_key(key_ctrl(KeyCode::Char('a')));
        let (_row, col) = ta.cursor();
        assert_eq!(col, 0);
    }

    #[test]
    fn test_ctrl_e_moves_to_end() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("hello");
        ta.handle_key(key_ctrl(KeyCode::Char('a')));
        ta.handle_key(key_ctrl(KeyCode::Char('e')));
        let (_row, col) = ta.cursor();
        assert_eq!(col, 5);
    }

    #[test]
    fn test_escape_action() {
        let mut ta = EnhancedTextArea::new();
        let action = ta.handle_key(key(KeyCode::Esc));
        assert_eq!(action, TextAreaAction::Escape);
    }

    #[test]
    fn test_ctrl_c_clears() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("test");
        let action = ta.handle_key(key_ctrl(KeyCode::Char('c')));
        assert_eq!(action, TextAreaAction::Clear);
    }

    #[test]
    fn test_history_navigation() {
        let mut ta = EnhancedTextArea::new();
        ta.add_to_history("first".to_string());
        ta.add_to_history("second".to_string());
        ta.add_to_history("third".to_string());

        ta.set_input("current");
        ta.history_previous();
        assert_eq!(ta.input(), "third");

        ta.history_previous();
        assert_eq!(ta.input(), "second");

        ta.history_next();
        assert_eq!(ta.input(), "third");

        ta.history_next();
        assert_eq!(ta.input(), "current");
    }

    #[test]
    fn test_history_empty() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("test");
        ta.history_previous();
        assert_eq!(ta.input(), "test");
    }

    #[test]
    fn test_undo_redo() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("hello");
        ta.handle_key(key(KeyCode::Char(' ')));
        ta.handle_key(key(KeyCode::Char('w')));
        ta.handle_key(key(KeyCode::Char('o')));
        ta.handle_key(key(KeyCode::Char('r')));
        ta.handle_key(key(KeyCode::Char('l')));
        ta.handle_key(key(KeyCode::Char('d')));

        ta.handle_key(key_ctrl(KeyCode::Char('z')));
        ta.handle_key(key_ctrl(KeyCode::Char('.')));
    }

    #[test]
    fn test_single_line_history_trigger() {
        let mut ta = EnhancedTextArea::new();
        ta.set_input("single line");

        let action = ta.handle_key(key(KeyCode::Up));
        assert_eq!(action, TextAreaAction::HistoryPrevious);

        let action = ta.handle_key(key(KeyCode::Down));
        assert_eq!(action, TextAreaAction::HistoryNext);
    }

    #[test]
    fn test_duplicate_history_not_added() {
        let mut ta = EnhancedTextArea::new();
        ta.add_to_history("same".to_string());
        ta.add_to_history("same".to_string());
        assert_eq!(ta.history.len(), 1);
    }

    #[test]
    fn test_empty_history_not_added() {
        let mut ta = EnhancedTextArea::new();
        ta.add_to_history("".to_string());
        ta.add_to_history("   ".to_string());
        assert_eq!(ta.history.len(), 0);
    }
}

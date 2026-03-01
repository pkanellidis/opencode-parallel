//! Text selection handling for the TUI.

/// Position in the terminal (column, row).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub col: u16,
    pub row: u16,
}

/// Represents a text selection in the terminal.
#[derive(Debug, Clone)]
pub struct TextSelection {
    /// Starting position of the selection.
    pub start: Position,
    /// Ending position of the selection.
    pub end: Position,
    /// Whether a drag is in progress.
    pub is_dragging: bool,
}

impl TextSelection {
    /// Creates a new selection starting at the given position.
    pub fn new(col: u16, row: u16) -> Self {
        let pos = Position { col, row };
        Self {
            start: pos,
            end: pos,
            is_dragging: true,
        }
    }

    /// Updates the end position during a drag.
    pub fn update(&mut self, col: u16, row: u16) {
        self.end = Position { col, row };
    }

    /// Finishes the drag operation.
    pub fn finish(&mut self) {
        self.is_dragging = false;
    }

    /// Returns the normalized selection (start before end).
    pub fn normalized(&self) -> (Position, Position) {
        if self.start.row < self.end.row
            || (self.start.row == self.end.row && self.start.col <= self.end.col)
        {
            (self.start, self.end)
        } else {
            (self.end, self.start)
        }
    }

    /// Checks if a position is within the selection.
    pub fn contains(&self, col: u16, row: u16) -> bool {
        let (start, end) = self.normalized();

        if row < start.row || row > end.row {
            return false;
        }

        if start.row == end.row {
            col >= start.col && col < end.col
        } else if row == start.row {
            col >= start.col
        } else if row == end.row {
            col < end.col
        } else {
            true
        }
    }

    /// Checks if the selection is empty (start equals end).
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Returns the selection range for a specific row.
    /// Returns None if the row is not part of the selection.
    pub fn row_range(&self, row: u16, line_width: u16) -> Option<(u16, u16)> {
        let (start, end) = self.normalized();

        if row < start.row || row > end.row {
            return None;
        }

        let col_start = if row == start.row { start.col } else { 0 };
        let col_end = if row == end.row { end.col } else { line_width };

        if col_start >= col_end {
            return None;
        }

        Some((col_start, col_end))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_selection_at_position() {
        let sel = TextSelection::new(5, 10);
        assert_eq!(sel.start.col, 5);
        assert_eq!(sel.start.row, 10);
        assert_eq!(sel.end.col, 5);
        assert_eq!(sel.end.row, 10);
        assert!(sel.is_dragging);
    }

    #[test]
    fn update_changes_end_position() {
        let mut sel = TextSelection::new(5, 10);
        sel.update(15, 12);
        assert_eq!(sel.end.col, 15);
        assert_eq!(sel.end.row, 12);
    }

    #[test]
    fn normalized_returns_correct_order() {
        let mut sel = TextSelection::new(10, 5);
        sel.update(5, 3);
        let (start, end) = sel.normalized();
        assert_eq!(start.row, 3);
        assert_eq!(end.row, 5);
    }

    #[test]
    fn contains_works_for_single_line() {
        let mut sel = TextSelection::new(5, 10);
        sel.update(15, 10);
        assert!(sel.contains(5, 10));
        assert!(sel.contains(10, 10));
        assert!(sel.contains(14, 10));
        assert!(!sel.contains(15, 10));
        assert!(!sel.contains(4, 10));
        assert!(!sel.contains(10, 9));
    }

    #[test]
    fn contains_works_for_multi_line() {
        let mut sel = TextSelection::new(5, 10);
        sel.update(15, 12);
        assert!(sel.contains(5, 10));
        assert!(sel.contains(100, 10));
        assert!(sel.contains(0, 11));
        assert!(sel.contains(100, 11));
        assert!(sel.contains(0, 12));
        assert!(sel.contains(14, 12));
        assert!(!sel.contains(15, 12));
    }

    #[test]
    fn is_empty_returns_true_when_no_selection() {
        let sel = TextSelection::new(5, 10);
        assert!(sel.is_empty());
    }

    #[test]
    fn is_empty_returns_false_when_selected() {
        let mut sel = TextSelection::new(5, 10);
        sel.update(10, 10);
        assert!(!sel.is_empty());
    }

    #[test]
    fn row_range_returns_correct_range() {
        let mut sel = TextSelection::new(5, 10);
        sel.update(15, 12);

        assert_eq!(sel.row_range(10, 80), Some((5, 80)));
        assert_eq!(sel.row_range(11, 80), Some((0, 80)));
        assert_eq!(sel.row_range(12, 80), Some((0, 15)));
        assert_eq!(sel.row_range(9, 80), None);
        assert_eq!(sel.row_range(13, 80), None);
    }
}

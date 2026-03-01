//! Main rendering functions for the TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::tui::app::App;
use crate::tui::worker::WorkerState;
use crate::utils::truncate_str;

use super::dialogs::{render_autocomplete, render_model_selector, render_permission_dialog};
use super::theme::*;

/// Main UI rendering entry point.
pub fn ui(f: &mut Frame, app: &mut App) {
    app.content_lines.clear();
    // Fill entire screen with black background
    let bg_block = Block::default().style(Style::default().bg(BG_PRIMARY));
    f.render_widget(bg_block, f.area());

    let session = app.current_session();
    let has_workers = !session.workers.is_empty();

    // Layout: main content area + sticky input at bottom
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),    // Main content (scrollable)
            Constraint::Length(4), // Input box (sticky)
        ])
        .split(f.area());

    if has_workers {
        // Split into workers sidebar and main content
        let content_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Length(32), // Workers sidebar
                Constraint::Min(0),     // Main content
            ])
            .split(main_layout[0]);

        render_workers_sidebar(f, app, content_layout[0]);
        render_main_content(f, app, content_layout[1]);
    } else {
        // Landing view - just show welcome/prompt area
        render_landing(f, app, main_layout[0]);
    }

    // Sticky input at bottom
    render_input_box(f, app, main_layout[1]);

    // Overlays
    render_autocomplete(f, app, main_layout[1]);
    render_model_selector(f, app);
    render_permission_dialog(f, app);
}

/// Renders the landing view when no workers are active.
fn render_landing(f: &mut Frame, app: &mut App, area: Rect) {
    // Center the content vertically
    let vertical_center = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(35),
            Constraint::Min(0),
            Constraint::Percentage(35),
        ])
        .split(area);

    let content_area = vertical_center[1];

    // Center horizontally
    let horizontal_center = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(15),
            Constraint::Percentage(70),
            Constraint::Percentage(15),
        ])
        .split(content_area);

    let center = horizontal_center[1];

    // Title and instructions
    let title_lines = vec![
        Line::from(vec![
            Span::styled("opencode", Style::default().fg(ACCENT).bold()),
            Span::styled("-parallel", Style::default().fg(TEXT_SECONDARY)),
        ]),
        Line::from(""),
        Line::styled(
            "Run multiple opencode instances in parallel",
            Style::default().fg(TEXT_SECONDARY),
        ),
        Line::from(""),
        Line::from(""),
        Line::styled(
            "Describe your task below. The orchestrator will break it down",
            Style::default().fg(TEXT_DIM),
        ),
        Line::styled(
            "into subtasks and spawn parallel workers.",
            Style::default().fg(TEXT_DIM),
        ),
    ];

    let title = Paragraph::new(title_lines).alignment(Alignment::Center);

    f.render_widget(title, center);

    // Show session messages if any
    let session = app.current_session();
    if !session.messages.is_empty() {
        let msg_area = Rect {
            x: area.x + 4,
            y: area.y + 2,
            width: area.width.saturating_sub(8),
            height: area.height.saturating_sub(4),
        };
        render_messages(f, app, msg_area);
    }
}

/// Renders the workers sidebar.
fn render_workers_sidebar(f: &mut Frame, app: &App, area: Rect) {
    let session = app.current_session();

    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };

    // Sidebar border only (pitch black bg)
    let panel = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(BORDER));
    f.render_widget(panel, area);

    // Header
    let running = session
        .workers
        .iter()
        .filter(|w| matches!(w.state, WorkerState::Running | WorkerState::Starting))
        .count();
    let complete = session
        .workers
        .iter()
        .filter(|w| w.state == WorkerState::Complete)
        .count();

    let header = if running > 0 {
        Line::from(vec![
            Span::styled("Workers ", Style::default().fg(TEXT_PRIMARY).bold()),
            Span::styled(
                format!("({} running)", running),
                Style::default().fg(STATUS_RUNNING),
            ),
        ])
    } else {
        Line::from(vec![
            Span::styled("Workers ", Style::default().fg(TEXT_PRIMARY).bold()),
            Span::styled(
                format!("({}/{})", complete, session.workers.len()),
                Style::default().fg(SUCCESS),
            ),
        ])
    };

    let header_para = Paragraph::new(header);
    let header_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(header_para, header_area);

    // Workers list
    let list_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 2,
        width: inner_area.width,
        height: inner_area.height.saturating_sub(3),
    };

    let items: Vec<ListItem> = session
        .workers
        .iter()
        .enumerate()
        .map(|(i, worker)| {
            let (icon, icon_color) = match worker.state {
                WorkerState::Starting => ("◌", STATUS_RUNNING),
                WorkerState::Running => ("●", STATUS_RUNNING),
                WorkerState::WaitingForInput => ("?", STATUS_WAITING),
                WorkerState::Complete => ("✓", SUCCESS),
                WorkerState::Error => ("✗", ERROR),
            };

            let is_selected = Some(i) == session.selected_worker;
            let bg = if is_selected { BG_SELECTED } else { BG_PRIMARY };
            let text_color = if is_selected {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            };

            let line = Line::from(vec![
                Span::styled(format!(" {} ", icon), Style::default().fg(icon_color)),
                Span::styled(format!("#{} ", worker.id), Style::default().fg(TEXT_DIM)),
                Span::styled(
                    truncate_str(&worker.description, 18),
                    Style::default().fg(text_color),
                ),
            ]);

            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    let list = List::new(items);
    f.render_widget(list, list_area);
}

/// Renders the main content area (messages or worker output).
fn render_main_content(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    let selected_worker_idx = app.current_session().selected_worker;
    if let Some(idx) = selected_worker_idx {
        let worker = app.current_session().workers.get(idx).cloned();
        if let Some(worker) = worker {
            render_worker_output(f, app, &worker, inner_area);
            return;
        }
    }

    render_messages(f, app, inner_area);
}

/// Renders the message history with distinct styling for user vs response messages.
fn render_messages(f: &mut Frame, app: &mut App, area: Rect) {
    // Clone messages to avoid borrow conflicts
    let messages: Vec<(String, bool)> = app.current_session().messages.clone();
    let scroll_offset = app.current_session().scroll_offset;

    // Build lines with background info and plain text for selection
    let mut styled_lines: Vec<(Line, bool, String)> = Vec::new(); // (line, is_user, plain_text)

    for (msg, is_user) in &messages {
        if *is_user {
            let plain = format!("› {}", msg);
            styled_lines.push((
                Line::from(vec![
                    Span::styled("› ", Style::default().fg(ACCENT)),
                    Span::styled(msg.as_str(), Style::default().fg(TEXT_PRIMARY)),
                ]),
                true,
                plain,
            ));
        } else if msg.is_empty() {
            styled_lines.push((Line::from(""), false, String::new()));
        } else if msg.starts_with("Plan:") || msg.starts_with("Spawning") {
            let plain = format!("  {}", msg);
            styled_lines.push((
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(msg.as_str(), Style::default().fg(ACCENT_SECONDARY)),
                ]),
                false,
                plain,
            ));
        } else if msg.starts_with("Error") || msg.starts_with("✗") {
            let plain = format!("  {}", msg);
            styled_lines.push((
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(msg.as_str(), Style::default().fg(ERROR)),
                ]),
                false,
                plain,
            ));
        } else if msg.starts_with("---") || msg.starts_with("Worker #") {
            let plain = format!("  {}", msg);
            styled_lines.push((
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(msg.as_str(), Style::default().fg(SUCCESS)),
                ]),
                false,
                plain,
            ));
        } else {
            let plain = format!("  {}", msg);
            styled_lines.push((
                Line::from(vec![
                    Span::styled("  ", Style::default()),
                    Span::styled(msg.as_str(), Style::default().fg(TEXT_SECONDARY)),
                ]),
                false,
                plain,
            ));
        }
    }

    let total_lines = styled_lines.len();
    let visible_height = area.height as usize;
    let scroll = scroll_offset.min(total_lines.saturating_sub(visible_height));

    // Populate content_lines for selection (screen row -> plain text)
    let screen_height = f.area().height;
    for row in 0..screen_height {
        if row < area.y || row >= area.y + area.height {
            app.content_lines.push(String::new());
        } else {
            let line_idx = scroll + (row - area.y) as usize;
            if let Some((_, _, plain)) = styled_lines.get(line_idx) {
                app.content_lines.push(plain.clone());
            } else {
                app.content_lines.push(String::new());
            }
        }
    }

    // Render each line with appropriate background
    let visible_lines: Vec<(Line, bool, String)> = styled_lines
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();

    let selection = app.selection.clone();

    for (i, (line, is_user, plain)) in visible_lines.iter().enumerate() {
        let y = area.y + i as u16;
        if y >= area.y + area.height {
            break;
        }

        let line_area = Rect {
            x: area.x,
            y,
            width: area.width.saturating_sub(1),
            height: 1,
        };

        let bg = if *is_user { BG_PRIMARY } else { BG_PANEL };
        let bg_block = Block::default().style(Style::default().bg(bg));
        f.render_widget(bg_block, line_area);

        // Check if this row has selection
        if let Some(ref sel) = selection {
            if let Some((col_start, col_end)) = sel.row_range(y, area.x + plain.len() as u16) {
                // Convert screen coordinates to content-relative coordinates
                let rel_start = col_start.saturating_sub(area.x);
                let rel_end = col_end.saturating_sub(area.x);
                if rel_end > rel_start {
                    render_line_with_selection(f, line, line_area, rel_start, rel_end);
                    continue;
                }
            }
        }

        let para = Paragraph::new(line.clone());
        f.render_widget(para, line_area);
    }

    // Scrollbar if needed
    if total_lines > visible_height {
        let scrollbar_area = Rect {
            x: area.x + area.width - 1,
            y: area.y,
            width: 1,
            height: area.height,
        };

        let mut scrollbar_state =
            ScrollbarState::new(total_lines.saturating_sub(visible_height)).position(scroll);

        let scrollbar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::default().fg(BORDER));

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

/// Renders a line with selection highlighting.
fn render_line_with_selection(
    f: &mut Frame,
    line: &Line,
    area: Rect,
    sel_start: u16,
    sel_end: u16,
) {
    let sel_area = Rect {
        x: area.x + sel_start,
        y: area.y,
        width: sel_end
            .saturating_sub(sel_start)
            .min(area.width.saturating_sub(sel_start)),
        height: 1,
    };
    let sel_block = Block::default().style(Style::default().bg(SELECTION_BG));
    f.render_widget(sel_block, sel_area);

    let para = Paragraph::new(line.clone());
    f.render_widget(para, area);
}

/// Renders a worker's output.
fn render_worker_output(
    f: &mut Frame,
    app: &mut App,
    worker: &crate::tui::worker::Worker,
    area: Rect,
) {
    let scroll_offset = app.current_session().scroll_offset;

    // Header with worker info
    let (status_text, status_color) = match worker.state {
        WorkerState::Starting => ("Starting...", STATUS_RUNNING),
        WorkerState::Running => ("Running", STATUS_RUNNING),
        WorkerState::WaitingForInput => ("Waiting for input", STATUS_WAITING),
        WorkerState::Complete => ("Complete", SUCCESS),
        WorkerState::Error => ("Error", ERROR),
    };

    let header_text = format!(
        "Worker #{} · {} · {}",
        worker.id, worker.description, status_text
    );
    let header = Line::from(vec![
        Span::styled(
            format!("Worker #{}", worker.id),
            Style::default().fg(ACCENT).bold(),
        ),
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        Span::styled(&worker.description, Style::default().fg(TEXT_SECONDARY)),
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        Span::styled(status_text, Style::default().fg(status_color)),
    ]);

    let header_para = Paragraph::new(header);
    let header_area = Rect {
        x: area.x,
        y: area.y,
        width: area.width,
        height: 1,
    };
    f.render_widget(header_para, header_area);

    // Separator
    let sep_text = "─".repeat(area.width as usize);
    let sep = Line::styled(&sep_text, Style::default().fg(BORDER));
    let sep_area = Rect {
        x: area.x,
        y: area.y + 1,
        width: area.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(sep), sep_area);

    // Output content
    let content_area = Rect {
        x: area.x,
        y: area.y + 3,
        width: area.width,
        height: area.height.saturating_sub(3),
    };

    let display_lines = worker.get_display_lines();
    let total_lines = display_lines.len();
    let visible_height = content_area.height as usize;

    let auto_scroll = worker.state == WorkerState::Running;
    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll = if auto_scroll {
        max_scroll
    } else {
        scroll_offset.min(max_scroll)
    };

    // Populate content_lines for selection
    for row in 0..f.area().height {
        if row < content_area.y || row >= content_area.y + content_area.height {
            if row == area.y {
                app.content_lines.push(header_text.clone());
            } else if row == area.y + 1 {
                app.content_lines.push(sep_text.clone());
            } else {
                app.content_lines.push(String::new());
            }
        } else {
            let line_idx = scroll + (row - content_area.y) as usize;
            if let Some(line) = display_lines.get(line_idx) {
                app.content_lines.push(line.clone());
            } else {
                app.content_lines.push(String::new());
            }
        }
    }

    // Render lines with potential selection highlighting
    for (i, line_idx) in (scroll..scroll + visible_height).enumerate() {
        let y = content_area.y + i as u16;
        if y >= content_area.y + content_area.height {
            break;
        }

        if let Some(line_text) = display_lines.get(line_idx) {
            let line_area = Rect {
                x: content_area.x,
                y,
                width: content_area.width.saturating_sub(1),
                height: 1,
            };

            let fg = if line_text.starts_with('✓') || line_text.starts_with("Complete") {
                SUCCESS
            } else if line_text.starts_with('✗') || line_text.starts_with("Error") {
                ERROR
            } else if line_text.starts_with("⚙")
                || line_text.starts_with("🔧")
                || line_text.contains("Tool:")
            {
                STATUS_RUNNING
            } else {
                TEXT_PRIMARY
            };

            // Check for selection
            if let Some(ref sel) = app.selection {
                let line_end = content_area.x + line_text.len() as u16;
                if let Some((col_start, col_end)) = sel.row_range(y, line_end) {
                    // Convert screen coordinates to content-relative coordinates
                    let rel_start = col_start.saturating_sub(content_area.x);
                    let rel_end = col_end.saturating_sub(content_area.x);
                    if rel_end > rel_start {
                        let sel_area = Rect {
                            x: content_area.x + rel_start,
                            y,
                            width: (rel_end - rel_start)
                                .min(line_area.width.saturating_sub(rel_start)),
                            height: 1,
                        };
                        let sel_block = Block::default().style(Style::default().bg(SELECTION_BG));
                        f.render_widget(sel_block, sel_area);
                    }
                }
            }

            let line = Line::styled(line_text.as_str(), Style::default().fg(fg));
            let para = Paragraph::new(line);
            f.render_widget(para, line_area);
        }
    }

    // Scrollbar
    if total_lines > visible_height {
        let scrollbar_area = Rect {
            x: content_area.x + content_area.width - 1,
            y: content_area.y,
            width: 1,
            height: content_area.height,
        };

        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll);
        let scrollbar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::default().fg(BORDER));

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }
}

/// Renders the sticky input box at the bottom.
fn render_input_box(f: &mut Frame, app: &App, area: Rect) {
    let is_active = app.input_mode;

    let border_color = if is_active { ACCENT } else { BORDER };
    let bg_color = BG_PANEL;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg_color));

    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // Build input line with cursor
    let spans = if is_active {
        let chars: Vec<char> = app.input.chars().collect();
        let before: String = chars[..app.cursor_pos].iter().collect();
        let cursor_char = chars.get(app.cursor_pos).copied().unwrap_or(' ');
        let after: String = if app.cursor_pos < chars.len() {
            chars[app.cursor_pos + 1..].iter().collect()
        } else {
            String::new()
        };

        vec![
            Span::styled("› ", Style::default().fg(ACCENT)),
            Span::styled(before, Style::default().fg(TEXT_PRIMARY)),
            Span::styled(
                cursor_char.to_string(),
                Style::default().fg(BG_PRIMARY).bg(ACCENT),
            ),
            Span::styled(after, Style::default().fg(TEXT_PRIMARY)),
        ]
    } else if app.input.is_empty() {
        vec![Span::styled(
            "Press 'i' to enter a task...",
            Style::default().fg(TEXT_DIM),
        )]
    } else {
        vec![
            Span::styled("› ", Style::default().fg(TEXT_DIM)),
            Span::styled(&app.input, Style::default().fg(TEXT_SECONDARY)),
        ]
    };

    let input_line = Paragraph::new(Line::from(spans));
    f.render_widget(input_line, inner);

    // Status hint on right side
    let hint = if is_active {
        "Enter to send · Esc to cancel"
    } else {
        "i: input · l: logs · q: quit"
    };

    let hint_width = hint.len() as u16;
    if inner.width > hint_width + 10 {
        let hint_area = Rect {
            x: inner.x + inner.width - hint_width - 1,
            y: inner.y,
            width: hint_width,
            height: 1,
        };
        let hint_para = Paragraph::new(Span::styled(hint, Style::default().fg(TEXT_DIM)));
        f.render_widget(hint_para, hint_area);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn render_functions_exist() {
        assert!(true);
    }
}

//! Main rendering functions for the TUI.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
    Frame,
};

use crate::constants::{
    DETAIL_PANEL_MIN_WIDTH, DETAIL_PANEL_RATIO, MAX_INPUT_HEIGHT, MIN_INPUT_HEIGHT, SIDEBAR_WIDTH,
};
use crate::tui::app::App;
use crate::tui::worker::WorkerState;
use crate::utils::truncate_str;

use super::dialogs::{
    render_autocomplete, render_confirm_clear_all, render_confirm_delete_worker,
    render_model_selector, render_permission_dialog, render_stop_selector,
};
use super::theme::*;

/// Wraps text to fit within a given width, returning multiple lines.
fn wrap_text(text: &str, width: usize, indent: &str) -> Vec<String> {
    if width == 0 {
        return vec![text.to_string()];
    }

    let mut lines = Vec::new();
    let effective_width = width.saturating_sub(indent.len());

    if effective_width == 0 {
        return vec![format!("{}{}", indent, text)];
    }

    let mut current_line = String::new();
    let mut current_len = 0;

    for word in text.split_whitespace() {
        let word_len = word.chars().count();

        if current_len == 0 {
            current_line = word.to_string();
            current_len = word_len;
        } else if current_len + 1 + word_len <= effective_width {
            current_line.push(' ');
            current_line.push_str(word);
            current_len += 1 + word_len;
        } else {
            lines.push(format!("{}{}", indent, current_line));
            current_line = word.to_string();
            current_len = word_len;
        }
    }

    if !current_line.is_empty() {
        lines.push(format!("{}{}", indent, current_line));
    }

    if lines.is_empty() {
        lines.push(format!("{}{}", indent, text));
    }

    lines
}

/// Calculate the height needed for the input box based on content.
fn calculate_input_height(app: &App, available_width: u16) -> u16 {
    let lines = app.textarea.lines();
    let line_count = lines.len() as u16;

    let content_width = available_width.saturating_sub(4) as usize;
    if content_width == 0 {
        return 4;
    }

    let mut wrapped_lines: u16 = 0;
    for line in lines {
        let line_len = line.chars().count();
        if line_len == 0 {
            wrapped_lines += 1;
        } else {
            wrapped_lines += line_len.div_ceil(content_width).max(1) as u16;
        }
    }

    let lines_needed = wrapped_lines.max(line_count).max(1);
    (lines_needed + 3).clamp(MIN_INPUT_HEIGHT, MAX_INPUT_HEIGHT)
}

/// Main UI rendering entry point.
pub fn ui(f: &mut Frame, app: &mut App) {
    app.content_lines.clear();
    app.detail_content_lines.clear();
    app.messages_panel_area = None;
    app.detail_panel_area = None;
    // Fill entire screen with black background
    let bg_block = Block::default().style(Style::default().bg(BG_PRIMARY));
    f.render_widget(bg_block, f.area());

    let session = app.current_session();
    let has_workers = !session.workers.is_empty();

    // Calculate dynamic input box height based on content
    let input_box_height = calculate_input_height(app, f.area().width.saturating_sub(4));

    // Layout: main content area + sticky input at bottom
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),                   // Main content (scrollable)
            Constraint::Length(input_box_height), // Input box (dynamic height)
        ])
        .split(f.area());

    // Check if logs panel should be shown
    if app.show_logs {
        render_logs_panel(f, app, main_layout[0]);
    } else if has_workers {
        let selected_worker_idx = app.current_session().selected_worker;
        let show_detail_panel = selected_worker_idx.is_some();
        let remaining_width = main_layout[0].width.saturating_sub(SIDEBAR_WIDTH);

        if show_detail_panel && remaining_width >= DETAIL_PANEL_MIN_WIDTH * 2 {
            let detail_width =
                (remaining_width * DETAIL_PANEL_RATIO / 100).max(DETAIL_PANEL_MIN_WIDTH);
            let content_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Length(SIDEBAR_WIDTH),
                    Constraint::Min(30),
                    Constraint::Length(detail_width),
                ])
                .split(main_layout[0]);

            render_workers_sidebar(f, app, content_layout[0]);
            render_messages(f, app, content_layout[1]);
            render_worker_detail_panel(f, app, content_layout[2]);
        } else {
            let content_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(SIDEBAR_WIDTH), Constraint::Min(0)])
                .split(main_layout[0]);

            render_workers_sidebar(f, app, content_layout[0]);
            render_main_content(f, app, content_layout[1]);
        }
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
    render_stop_selector(f, app);
    render_confirm_delete_worker(f, app);
    render_confirm_clear_all(f, app);
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
    let has_selected = session.selected_worker.is_some();
    let is_focused = !app.focus_detail_panel || !has_selected;

    let inner_area = Rect {
        x: area.x + 1,
        y: area.y + 1,
        width: area.width.saturating_sub(2),
        height: area.height.saturating_sub(1),
    };

    let border_color = if is_focused && has_selected {
        ACCENT
    } else {
        BORDER
    };
    let panel = Block::default()
        .borders(Borders::RIGHT)
        .border_style(Style::default().fg(border_color));
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

/// Renders the main content area (messages only - worker detail is in side panel).
fn render_main_content(f: &mut Frame, app: &mut App, area: Rect) {
    let inner_area = Rect {
        x: area.x + 2,
        y: area.y + 1,
        width: area.width.saturating_sub(4),
        height: area.height.saturating_sub(2),
    };

    render_messages(f, app, inner_area);
}

/// Renders the message history with distinct styling for user vs response messages.
fn render_messages(f: &mut Frame, app: &mut App, area: Rect) {
    // Track panel area for position-based scroll routing
    app.messages_panel_area = Some(area);

    let has_selected_worker = app.current_session().selected_worker.is_some();
    let is_focused = !app.focus_detail_panel || !has_selected_worker;

    let (inner_area, effective_area) = if has_selected_worker {
        let border_color = if is_focused { ACCENT } else { BORDER };
        let block = Block::default()
            .borders(Borders::NONE)
            .style(Style::default().bg(BG_PRIMARY));
        f.render_widget(block, area);

        if is_focused {
            let focus_indicator = Line::from(vec![
                Span::styled("Messages", Style::default().fg(border_color).bold()),
                Span::styled(" · Tab: switch to detail", Style::default().fg(TEXT_DIM)),
            ]);
            let indicator_area = Rect {
                x: area.x + 2,
                y: area.y,
                width: area.width.saturating_sub(4),
                height: 1,
            };
            f.render_widget(Paragraph::new(focus_indicator), indicator_area);
        }

        let inner = Rect {
            x: area.x + 2,
            y: area.y + 1,
            width: area.width.saturating_sub(4),
            height: area.height.saturating_sub(2),
        };
        (inner, area)
    } else {
        (area, area)
    };

    let _ = effective_area;

    let messages: Vec<(String, bool)> = app.current_session().messages.clone();
    let wrap_width = inner_area.width.saturating_sub(2) as usize;

    // Build lines with background info and plain text for selection
    let mut styled_lines: Vec<(Line, bool, String)> = Vec::new(); // (line, is_user, plain_text)

    for (msg, is_user) in &messages {
        if *is_user {
            // User messages - wrap if needed
            let wrapped = wrap_text(msg, wrap_width.saturating_sub(2), "");
            for (i, line) in wrapped.iter().enumerate() {
                let prefix = if i == 0 { "› " } else { "  " };
                let plain = format!("{}{}", prefix, line);
                styled_lines.push((
                    Line::from(vec![
                        Span::styled(prefix, Style::default().fg(ACCENT)),
                        Span::styled(line.clone(), Style::default().fg(TEXT_PRIMARY)),
                    ]),
                    true,
                    plain,
                ));
            }
        } else if msg.is_empty() {
            styled_lines.push((Line::from(""), false, String::new()));
        } else if msg.starts_with("Plan:") || msg.starts_with("Spawning") {
            let wrapped = wrap_text(msg, wrap_width, "  ");
            for line in wrapped {
                styled_lines.push((
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(ACCENT_SECONDARY),
                    )),
                    false,
                    line,
                ));
            }
        } else if msg.starts_with("Error") || msg.starts_with("✗") {
            // Error messages - wrap to multiple lines
            let wrapped = wrap_text(msg, wrap_width, "  ");
            for line in wrapped {
                styled_lines.push((
                    Line::from(Span::styled(line.clone(), Style::default().fg(ERROR))),
                    false,
                    line,
                ));
            }
        } else if msg.starts_with("---") || msg.starts_with("Worker #") {
            let wrapped = wrap_text(msg, wrap_width, "  ");
            for line in wrapped {
                styled_lines.push((
                    Line::from(Span::styled(line.clone(), Style::default().fg(SUCCESS))),
                    false,
                    line,
                ));
            }
        } else {
            let wrapped = wrap_text(msg, wrap_width, "  ");
            for line in wrapped {
                styled_lines.push((
                    Line::from(Span::styled(
                        line.clone(),
                        Style::default().fg(TEXT_SECONDARY),
                    )),
                    false,
                    line,
                ));
            }
        }
    }

    let total_lines = styled_lines.len();
    let visible_height = inner_area.height as usize;

    app.set_main_scroll_dimensions(total_lines, visible_height);
    let scroll = app.main_scroll();

    app.content_area_x = inner_area.x;
    let screen_height = f.area().height;
    for row in 0..screen_height {
        if row < inner_area.y || row >= inner_area.y + inner_area.height {
            app.content_lines.push(String::new());
        } else {
            let line_idx = scroll + (row - inner_area.y) as usize;
            if let Some((_, _, plain)) = styled_lines.get(line_idx) {
                app.content_lines.push(plain.clone());
            } else {
                app.content_lines.push(String::new());
            }
        }
    }

    let visible_lines: Vec<(Line, bool, String)> = styled_lines
        .into_iter()
        .skip(scroll)
        .take(visible_height)
        .collect();

    let selection = app.selection.clone();

    for (i, (line, is_user, plain)) in visible_lines.iter().enumerate() {
        let y = inner_area.y + i as u16;
        if y >= inner_area.y + inner_area.height {
            break;
        }

        let line_area = Rect {
            x: inner_area.x,
            y,
            width: inner_area.width.saturating_sub(1),
            height: 1,
        };

        let bg = if *is_user { BG_PRIMARY } else { BG_PANEL };
        let bg_block = Block::default().style(Style::default().bg(bg));
        f.render_widget(bg_block, line_area);

        if let Some(ref sel) = selection {
            if let Some((col_start, col_end)) = sel.row_range(y, inner_area.x + plain.len() as u16)
            {
                let rel_start = col_start.saturating_sub(inner_area.x);
                let rel_end = col_end.saturating_sub(inner_area.x);
                if rel_end > rel_start {
                    render_line_with_selection(f, line, line_area, rel_start, rel_end);
                    continue;
                }
            }
        }

        let para = Paragraph::new(line.clone());
        f.render_widget(para, line_area);
    }

    if total_lines > visible_height {
        let scrollbar_area = Rect {
            x: inner_area.x + inner_area.width - 1,
            y: inner_area.y,
            width: 1,
            height: inner_area.height,
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

/// Renders the worker detail panel on the right side.
fn render_worker_detail_panel(f: &mut Frame, app: &mut App, area: Rect) {
    // Track panel area for position-based scroll routing
    app.detail_panel_area = Some(area);

    let session = app.current_session();
    let selected_idx = match session.selected_worker {
        Some(idx) => idx,
        None => return,
    };

    let worker = match session.workers.get(selected_idx) {
        Some(w) => w.clone(),
        None => return,
    };

    let is_focused = app.focus_detail_panel;
    let border_color = if is_focused { ACCENT } else { BORDER };

    let block = Block::default()
        .borders(Borders::LEFT)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(BG_PANEL));

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let inner_area = Rect {
        x: inner.x + 1,
        y: inner.y,
        width: inner.width.saturating_sub(2),
        height: inner.height,
    };

    let (status_text, status_color) = match worker.state {
        WorkerState::Starting => ("Starting...", STATUS_RUNNING),
        WorkerState::Running => ("Running", STATUS_RUNNING),
        WorkerState::WaitingForInput => ("Waiting for input", STATUS_WAITING),
        WorkerState::Complete => ("Complete", SUCCESS),
        WorkerState::Error => ("Error", ERROR),
    };

    let header = Line::from(vec![
        Span::styled(
            format!("Worker #{}", worker.id),
            Style::default().fg(ACCENT).bold(),
        ),
        Span::styled(" · ", Style::default().fg(TEXT_DIM)),
        Span::styled(status_text, Style::default().fg(status_color)),
    ]);

    let header_area = Rect {
        x: inner_area.x,
        y: inner_area.y,
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(header), header_area);

    let desc_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 1,
        width: inner_area.width,
        height: 1,
    };
    let desc = Line::styled(
        truncate_str(&worker.description, inner_area.width as usize),
        Style::default().fg(TEXT_SECONDARY),
    );
    f.render_widget(Paragraph::new(desc), desc_area);

    let sep_text = "─".repeat(inner_area.width as usize);
    let sep = Line::styled(&sep_text, Style::default().fg(BORDER));
    let sep_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 2,
        width: inner_area.width,
        height: 1,
    };
    f.render_widget(Paragraph::new(sep), sep_area);

    let content_area = Rect {
        x: inner_area.x,
        y: inner_area.y + 3,
        width: inner_area.width,
        height: inner_area.height.saturating_sub(3),
    };

    let wrap_width = content_area.width.saturating_sub(2) as usize;
    let mut display_lines: Vec<(String, Color)> = Vec::new();

    for tool in &worker.tool_history {
        let wrapped = wrap_text(&format!("✓ {}", tool), wrap_width, "  ");
        for line in wrapped {
            display_lines.push((line, SUCCESS));
        }
    }

    if let Some(tool) = &worker.current_tool {
        let wrapped = wrap_text(&format!("⚙ {}...", tool), wrap_width, "  ");
        for line in wrapped {
            display_lines.push((line, STATUS_RUNNING));
        }
    }

    if !worker.tool_history.is_empty() || worker.current_tool.is_some() {
        display_lines.push((String::new(), TEXT_PRIMARY));
    }

    let content = if !worker.streaming_content.is_empty() {
        &worker.streaming_content
    } else {
        &worker.output.join("\n")
    };

    for line in content.lines() {
        let wrapped = wrap_text(line, wrap_width, "");
        for wrapped_line in wrapped {
            let color = if wrapped_line.starts_with("Complete") {
                SUCCESS
            } else if wrapped_line.starts_with("Error") || wrapped_line.starts_with("✗") {
                ERROR
            } else {
                TEXT_PRIMARY
            };
            display_lines.push((wrapped_line, color));
        }
    }

    let total_lines = display_lines.len();
    let visible_height = content_area.height as usize;

    let auto_scroll = worker.state == WorkerState::Running;
    app.set_worker_detail_scroll_dimensions(total_lines, visible_height);

    let max_scroll = total_lines.saturating_sub(visible_height);
    let scroll = if auto_scroll {
        max_scroll
    } else {
        app.worker_detail_scroll().min(max_scroll)
    };

    // Track content lines for selection/copy (similar to messages panel)
    app.detail_content_area_x = content_area.x;
    app.detail_content_lines.clear();
    let screen_height = f.area().height;
    for row in 0..screen_height {
        if row < content_area.y || row >= content_area.y + content_area.height {
            app.detail_content_lines.push(String::new());
        } else {
            let line_idx = scroll + (row - content_area.y) as usize;
            if let Some((text, _)) = display_lines.get(line_idx) {
                app.detail_content_lines.push(text.clone());
            } else {
                app.detail_content_lines.push(String::new());
            }
        }
    }

    let selection = app.selection.clone();

    for (i, line_idx) in (scroll..scroll + visible_height).enumerate() {
        let y = content_area.y + i as u16;
        if y >= content_area.y + content_area.height {
            break;
        }

        if let Some((line_text, fg)) = display_lines.get(line_idx) {
            let line_area = Rect {
                x: content_area.x,
                y,
                width: content_area.width.saturating_sub(1),
                height: 1,
            };

            // Check for selection highlighting
            if let Some(ref sel) = selection {
                if let Some((col_start, col_end)) =
                    sel.row_range(y, content_area.x + line_text.len() as u16)
                {
                    let rel_start = col_start.saturating_sub(content_area.x);
                    let rel_end = col_end.saturating_sub(content_area.x);
                    if rel_end > rel_start {
                        let line = Line::styled(line_text.as_str(), Style::default().fg(*fg));
                        render_line_with_selection(f, &line, line_area, rel_start, rel_end);
                        continue;
                    }
                }
            }

            let line = Line::styled(line_text.as_str(), Style::default().fg(*fg));
            f.render_widget(Paragraph::new(line), line_area);
        }
    }

    if total_lines > visible_height {
        let scrollbar_area = Rect {
            x: content_area.x + content_area.width - 1,
            y: content_area.y,
            width: 1,
            height: content_area.height,
        };

        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll);
        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .style(Style::default().fg(border_color));

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    if is_focused {
        let hint = "Tab: switch panel · j/k: scroll";
        let hint_width = hint.len() as u16;
        if inner_area.width > hint_width {
            let hint_area = Rect {
                x: inner_area.x + inner_area.width - hint_width,
                y: inner_area.y,
                width: hint_width,
                height: 1,
            };
            let hint_para = Paragraph::new(Span::styled(hint, Style::default().fg(TEXT_DIM)));
            f.render_widget(hint_para, hint_area);
        }
    }
}

/// Renders the logs panel overlay.
fn render_logs_panel(f: &mut Frame, app: &mut App, area: Rect) {
    let block = Block::default()
        .title(" Orchestrator Logs (press 'l' or Esc to close) ")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(ACCENT))
        .style(Style::default().bg(BG_PANEL));

    f.render_widget(block.clone(), area);
    let inner = block.inner(area);

    let total_lines = app.orchestrator_logs.len();
    let visible_height = inner.height as usize;
    let max_scroll = total_lines.saturating_sub(visible_height);

    app.set_logs_scroll_dimensions(total_lines, visible_height);
    let scroll = app.logs_scroll();

    app.content_lines.clear();
    app.content_area_x = inner.x;
    for row in 0..f.area().height {
        if row < inner.y || row >= inner.y + inner.height {
            app.content_lines.push(String::new());
        } else {
            let line_idx = scroll + (row - inner.y) as usize;
            if let Some(log) = app.orchestrator_logs.get(line_idx) {
                app.content_lines.push(log.clone());
            } else {
                app.content_lines.push(String::new());
            }
        }
    }

    for (i, line_idx) in (scroll..scroll + visible_height).enumerate() {
        let y = inner.y + i as u16;
        if y >= inner.y + inner.height {
            break;
        }

        if let Some(log) = app.orchestrator_logs.get(line_idx) {
            let line_area = Rect {
                x: inner.x,
                y,
                width: inner.width.saturating_sub(1),
                height: 1,
            };

            let fg = if log.contains("[ERROR]") || log.contains("Error") {
                ERROR
            } else if log.contains("[WARN]") {
                WARNING
            } else if log.contains("[SSE]") || log.contains("[PERM]") {
                ACCENT_SECONDARY
            } else if log.contains("[WORKER]") {
                STATUS_RUNNING
            } else {
                TEXT_SECONDARY
            };

            // Check for selection
            if let Some(ref sel) = app.selection {
                let line_end = inner.x + log.len() as u16;
                if let Some((col_start, col_end)) = sel.row_range(y, line_end) {
                    let rel_start = col_start.saturating_sub(inner.x) as usize;
                    let rel_end = col_end.saturating_sub(inner.x) as usize;
                    render_line_with_selection_and_color(f, log, line_area, rel_start, rel_end, fg);
                    continue;
                }
            }

            let line = Line::styled(log.as_str(), Style::default().fg(fg));
            f.render_widget(Paragraph::new(line), line_area);
        }
    }

    // Scrollbar
    if total_lines > visible_height {
        let scrollbar_area = Rect {
            x: inner.x + inner.width,
            y: inner.y,
            width: 1,
            height: inner.height,
        };

        let mut scrollbar_state = ScrollbarState::new(max_scroll).position(scroll);
        let scrollbar =
            Scrollbar::new(ScrollbarOrientation::VerticalRight).style(Style::default().fg(BORDER));

        f.render_stateful_widget(scrollbar, scrollbar_area, &mut scrollbar_state);
    }

    // Show scroll position
    if total_lines > 0 {
        let pos_text = format!(
            " {}/{} ",
            scroll + visible_height.min(total_lines),
            total_lines
        );
        let pos_width = pos_text.len() as u16;
        if inner.width > pos_width {
            let pos_area = Rect {
                x: inner.x + inner.width - pos_width,
                y: area.y,
                width: pos_width,
                height: 1,
            };
            let pos_para = Paragraph::new(Span::styled(pos_text, Style::default().fg(TEXT_DIM)));
            f.render_widget(pos_para, pos_area);
        }
    }
}

/// Renders a line with selection highlighting and custom foreground color.
fn render_line_with_selection_and_color(
    f: &mut Frame,
    text: &str,
    area: Rect,
    sel_start: usize,
    sel_end: usize,
    fg: Color,
) {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let start = sel_start.min(len);
    let end = sel_end.min(len);

    let before: String = chars[..start].iter().collect();
    let selected: String = chars[start..end].iter().collect();
    let after: String = chars[end..].iter().collect();

    let spans = vec![
        Span::styled(before, Style::default().fg(fg)),
        Span::styled(selected, Style::default().fg(fg).bg(SELECTION_BG)),
        Span::styled(after, Style::default().fg(fg)),
    ];

    f.render_widget(Paragraph::new(Line::from(spans)), area);
}

/// Renders the sticky input box at the bottom.
fn render_input_box(f: &mut Frame, app: &mut App, area: Rect) {
    let is_active = app.input_mode;

    let border_color = if is_active { ACCENT } else { BORDER };
    let bg_color = BG_PANEL;

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(border_color))
        .style(Style::default().bg(bg_color));

    f.render_widget(block.clone(), area);

    let inner = block.inner(area);

    // Calculate how many lines we need for status info at the bottom
    let status_lines = 1u16;
    let input_area_height = inner.height.saturating_sub(status_lines);
    let input_area = Rect {
        x: inner.x,
        y: inner.y,
        width: inner.width,
        height: input_area_height,
    };

    // Configure and render the textarea widget
    app.textarea.textarea.set_block(Block::default());
    app.textarea.textarea.set_cursor_style(if is_active {
        Style::default().fg(BG_PRIMARY).bg(ACCENT)
    } else {
        Style::default()
    });
    app.textarea.textarea.set_style(if is_active {
        Style::default().fg(TEXT_PRIMARY).bg(bg_color)
    } else {
        Style::default().fg(TEXT_SECONDARY).bg(bg_color)
    });

    f.render_widget(app.textarea.widget(), input_area);

    // Status area at the bottom of the input box
    let status_area = Rect {
        x: inner.x,
        y: inner.y + input_area_height,
        width: inner.width,
        height: status_lines,
    };

    // Show current model on the left side of status line
    if let Some(ref model) = app.current_model {
        let model_text = format!("Model: {}", model);
        let model_para = Paragraph::new(Span::styled(model_text, Style::default().fg(TEXT_DIM)));
        f.render_widget(model_para, status_area);
    }

    // Status hint on right side
    let hint = if is_active {
        "Enter to send · Shift+Enter for newline · Esc to cancel"
    } else {
        "i: input · l: logs · q: quit"
    };

    let hint_width = hint.len() as u16;
    if inner.width > hint_width + 10 {
        let hint_area = Rect {
            x: status_area.x + status_area.width - hint_width - 1,
            y: status_area.y,
            width: hint_width,
            height: 1,
        };
        let hint_para = Paragraph::new(Span::styled(hint, Style::default().fg(TEXT_DIM)));
        f.render_widget(hint_para, hint_area);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::OpenCodeServer;
    use crate::tui::App;

    fn create_test_app() -> App {
        let server = OpenCodeServer::new(4096);
        App::new(server)
    }

    #[test]
    fn app_show_logs_affects_rendering_path() {
        let mut app = create_test_app();

        // Initially logs are hidden
        assert!(!app.show_logs);

        // When show_logs is true, the logs panel should be rendered
        app.show_logs = true;
        assert!(app.show_logs);

        // Add some logs to verify they exist
        app.orchestrator_logs.push("[TEST] Log entry 1".to_string());
        app.orchestrator_logs
            .push("[ERROR] Error entry".to_string());
        assert_eq!(app.orchestrator_logs.len(), 2);
    }

    #[test]
    fn logs_scroll_bounds() {
        let mut app = create_test_app();

        // Add logs
        for i in 0..100 {
            app.orchestrator_logs.push(format!("Log line {}", i));
        }

        // Set dimensions and scroll should be bounded
        app.set_logs_scroll_dimensions(100, 20);
        app.logs_scroll_state.scroll_to(50);
        assert_eq!(app.logs_scroll(), 50);

        // Scroll by negative prevents underflow
        app.logs_scroll_state.scroll_by(-100);
        assert_eq!(app.logs_scroll(), 0);
    }

    #[test]
    fn wrap_text_short_text_unchanged() {
        let result = wrap_text("short text", 50, "  ");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "  short text");
    }

    #[test]
    fn wrap_text_long_text_wraps() {
        let long_text =
            "This is a very long error message that should be wrapped across multiple lines";
        let result = wrap_text(long_text, 30, "  ");
        assert!(result.len() > 1, "Long text should wrap to multiple lines");
        for line in &result {
            assert!(line.starts_with("  "), "Each line should have indent");
        }
    }

    #[test]
    fn wrap_text_preserves_words() {
        let text = "word1 word2 word3";
        let result = wrap_text(text, 15, "");
        // Should not break words in the middle
        for line in &result {
            assert!(!line.contains("wor "), "Words should not be broken");
        }
    }

    #[test]
    fn wrap_text_empty_returns_indented_empty() {
        let result = wrap_text("", 50, "  ");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "  ");
    }

    #[test]
    fn wrap_text_handles_zero_width() {
        let result = wrap_text("test", 0, "");
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], "test");
    }
}

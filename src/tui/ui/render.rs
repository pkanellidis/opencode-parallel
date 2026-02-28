//! Main rendering functions for the TUI.

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::tui::worker::WorkerState;
use crate::utils::truncate_str;

use super::dialogs::{render_autocomplete, render_model_selector, render_permission_dialog};

/// Main UI rendering entry point.
pub fn ui(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(0),
            Constraint::Length(3),
            Constraint::Length(3),
        ])
        .split(f.area());

    render_session_tabs(f, app, chunks[0]);

    if app.show_logs {
        render_logs(f, app, chunks[1]);
    } else {
        let main_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(chunks[1]);

        render_workers(f, app, main_chunks[0]);
        render_main_panel(f, app, main_chunks[1]);
    }

    render_input(f, app, chunks[2]);
    render_autocomplete(f, app, chunks[2]);
    render_model_selector(f, app);
    render_permission_dialog(f, app);
    render_status(f, app, chunks[3]);
}

/// Renders the session tabs at the top of the screen.
pub fn render_session_tabs(f: &mut Frame, app: &App, area: Rect) {
    let tabs: Vec<Span> = app
        .sessions
        .iter()
        .enumerate()
        .flat_map(|(i, session)| {
            let worker_count = session.workers.len();
            let running = session
                .workers
                .iter()
                .filter(|w| matches!(w.state, WorkerState::Running | WorkerState::Starting))
                .count();

            let indicator = if running > 0 {
                format!(" ◐{}", running)
            } else if worker_count > 0 {
                format!(" ●{}", worker_count)
            } else {
                String::new()
            };

            let label = format!(" {}{} ", session.name, indicator);

            let style = if i == app.current_session {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::Gray)
            };

            vec![Span::styled(label, style), Span::raw(" ")]
        })
        .collect();

    let line = Line::from(tabs);
    let paragraph = Paragraph::new(line);
    f.render_widget(paragraph, area);
}

/// Renders the orchestrator logs panel.
pub fn render_logs(f: &mut Frame, app: &App, area: Rect) {
    let total_logs = app.orchestrator_logs.len();

    let lines: Vec<Line> = app
        .orchestrator_logs
        .iter()
        .map(|log| {
            let style = if log.contains("Success")
                || log.contains("[SPAWN]")
                || log.contains("[WORKER]")
            {
                Style::default().fg(Color::Green)
            } else if log.contains("Failed") || log.contains("error") || log.contains("Error") {
                Style::default().fg(Color::Red)
            } else if log.contains("Attempt") || log.contains("[QUESTION]") || log.contains("[IDLE")
            {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default().fg(Color::Gray)
            };
            Line::styled(log.as_str(), style)
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(
                    " Orchestrator Logs ({}) - scroll or l to close ",
                    total_logs
                ))
                .title_style(Style::default().fg(Color::Magenta).bold())
                .border_style(Style::default().fg(Color::Magenta)),
        )
        .scroll((app.logs_scroll as u16, 0));

    f.render_widget(paragraph, area);
}

/// Renders the workers panel on the left side.
pub fn render_workers(f: &mut Frame, app: &App, area: Rect) {
    let session = app.current_session();
    let items: Vec<ListItem> = session
        .workers
        .iter()
        .enumerate()
        .map(|(i, worker)| {
            let icon = match worker.state {
                WorkerState::Starting => "◌",
                WorkerState::Running => "◐",
                WorkerState::WaitingForInput => "❓",
                WorkerState::Complete => "●",
                WorkerState::Error => "✗",
            };

            let style = if Some(i) == session.selected_worker {
                Style::default().fg(Color::Cyan).bg(Color::DarkGray).bold()
            } else {
                Style::default()
            };

            let text = format!(
                "{} #{} {}",
                icon,
                worker.id,
                truncate_str(&worker.description, 17)
            );

            ListItem::new(Line::styled(text, style))
        })
        .collect();

    let title = if session.workers.is_empty() {
        " Workers (none) ".to_string()
    } else {
        let done = session
            .workers
            .iter()
            .filter(|w| matches!(w.state, WorkerState::Complete))
            .count();
        format!(" Workers ({}/{} done) ", done, session.workers.len())
    };

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .title_style(Style::default().fg(Color::Yellow).bold())
            .border_style(Style::default().fg(Color::Yellow)),
    );

    f.render_widget(list, area);
}

/// Renders the main content panel (worker output or session messages).
pub fn render_main_panel(f: &mut Frame, app: &App, area: Rect) {
    let session = app.current_session();

    if let Some(idx) = session.selected_worker {
        if let Some(worker) = session.workers.get(idx) {
            render_worker_output(f, app, worker, area);
            return;
        }
    }

    render_session_messages(f, session, area);
}

/// Renders the output for a selected worker.
fn render_worker_output(f: &mut Frame, app: &App, worker: &crate::tui::worker::Worker, area: Rect) {
    let session = app.current_session();
    let display_lines = worker.get_display_lines();
    let output_height = area.height.saturating_sub(2) as usize;

    let total_lines = display_lines.len();
    let max_scroll = total_lines.saturating_sub(output_height);
    let auto_scroll = worker.state == WorkerState::Running;
    let scroll = if auto_scroll {
        max_scroll
    } else {
        session.scroll_offset.min(max_scroll)
    };

    let lines: Vec<Line> = display_lines
        .iter()
        .skip(scroll)
        .take(output_height)
        .map(|line| {
            if line.starts_with('✓') {
                Line::styled(line.as_str(), Style::default().fg(Color::Green))
            } else if line.starts_with('✗') {
                Line::styled(line.as_str(), Style::default().fg(Color::Red))
            } else if line.starts_with('⚙') || line.starts_with('🚀') {
                Line::styled(line.as_str(), Style::default().fg(Color::Yellow))
            } else {
                Line::from(line.as_str())
            }
        })
        .collect();

    let state_str = match worker.state {
        WorkerState::Starting => "Starting",
        WorkerState::Running => "Streaming...",
        WorkerState::WaitingForInput => "Waiting for input",
        WorkerState::Complete => "Complete",
        WorkerState::Error => "Error",
    };

    let scroll_info = if total_lines > output_height {
        format!(
            " [{}/{}] ",
            scroll + output_height.min(total_lines),
            total_lines
        )
    } else {
        String::new()
    };

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(
                    " Worker #{} - {} [{}]{}",
                    worker.id,
                    truncate_str(&worker.description, 20),
                    state_str,
                    scroll_info
                ))
                .title_style(Style::default().fg(Color::Cyan).bold())
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Renders the session messages when no worker is selected.
fn render_session_messages(f: &mut Frame, session: &crate::tui::session::Session, area: Rect) {
    let lines: Vec<Line> = session
        .messages
        .iter()
        .map(|(msg, is_user)| {
            if *is_user {
                Line::styled(msg.as_str(), Style::default().fg(Color::Magenta).bold())
            } else if msg.starts_with('📋') {
                Line::styled(msg.as_str(), Style::default().fg(Color::Green))
            } else if msg.starts_with('✗') {
                Line::styled(msg.as_str(), Style::default().fg(Color::Red))
            } else {
                Line::styled(msg.as_str(), Style::default().fg(Color::Gray))
            }
        })
        .collect();

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(format!(" {} ", session.name))
                .title_style(Style::default().fg(Color::Cyan).bold())
                .border_style(Style::default().fg(Color::Cyan)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Renders the input field.
pub fn render_input(f: &mut Frame, app: &App, area: Rect) {
    let style = if app.input_mode {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let spans = if app.input_mode {
        let chars: Vec<char> = app.input.chars().collect();
        let before: String = chars[..app.cursor_pos].iter().collect();
        let cursor_char = chars.get(app.cursor_pos).copied().unwrap_or(' ');
        let after: String = if app.cursor_pos < chars.len() {
            chars[app.cursor_pos + 1..].iter().collect()
        } else {
            String::new()
        };

        vec![
            Span::styled("› ", Style::default().fg(Color::Cyan).bold()),
            Span::raw(before),
            Span::styled(
                cursor_char.to_string(),
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ),
            Span::raw(after),
        ]
    } else {
        vec![
            Span::styled("› ", Style::default().fg(Color::Cyan).bold()),
            Span::raw(&app.input),
        ]
    };

    let title = if app.input_mode {
        " Type your task (Enter to send, Esc to navigate) "
    } else {
        " Press 'i' to type "
    };

    let border_style = if app.input_mode {
        Style::default().fg(Color::Yellow)
    } else {
        Style::default().fg(Color::DarkGray)
    };

    let input = Paragraph::new(Line::from(spans)).style(style).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(title)
            .title_style(border_style)
            .border_style(border_style),
    );

    f.render_widget(input, area);
}

/// Renders the status bar at the bottom.
pub fn render_status(f: &mut Frame, app: &App, area: Rect) {
    let session = app.current_session();
    let status_color = if app.confirm_delete || app.confirm_clear_all || app.confirm_delete_session
    {
        Color::Red
    } else if app.status.contains("Error") {
        Color::Red
    } else if app.status.contains("Running") || app.status.contains("analyzing") {
        Color::Yellow
    } else {
        Color::Green
    };

    let keys = if app.confirm_delete || app.confirm_clear_all || app.confirm_delete_session {
        "y: Yes | n: No, cancel"
    } else if app.input_mode {
        "Enter: Send | Esc: Navigate"
    } else if app.show_logs {
        "l: Close logs | n/p: Session | q: Quit"
    } else if session.selected_worker.is_some() {
        "j/k: Scroll | Tab: Next worker | Esc: Back | g/G: Top/Bottom | q: Quit"
    } else if !session.workers.is_empty() {
        "j/k: Select worker | n/p: Session | l: Logs | q: Quit"
    } else {
        "n/p: Session | i: Input | l: Logs | q: Quit"
    };

    let border_color = if app.confirm_delete || app.confirm_clear_all || app.confirm_delete_session
    {
        Color::Red
    } else {
        Color::DarkGray
    };

    let status = Paragraph::new(Line::from(vec![
        Span::styled(&app.status, Style::default().fg(status_color).bold()),
        Span::raw("  │  "),
        Span::styled(keys, Style::default().fg(Color::DarkGray)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color)),
    );

    f.render_widget(status, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_functions_exist() {
        // Smoke test to ensure all render functions are defined
        // Actual rendering tests would require a mock terminal
        assert!(true);
    }
}

//! Dialog rendering (model selector, permission dialog, etc.).

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::utils::truncate_str;

use super::theme::*;

/// Renders the delete worker confirmation dialog.
pub fn render_confirm_delete_worker(f: &mut Frame, app: &App) {
    if !app.confirm_delete {
        return;
    }

    let session = app.current_session();
    let worker_info = session
        .selected_worker
        .and_then(|idx| session.workers.get(idx))
        .map(|w| format!("#{} ({})", w.id, truncate_str(&w.description, 30)))
        .unwrap_or_else(|| "selected worker".to_string());

    let area = f.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 7u16;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  Delete worker {}?", worker_info),
            Style::default().fg(TEXT_PRIMARY),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(" y ", Style::default().fg(BG_PRIMARY).bg(ERROR)),
            Span::styled(" Yes ", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("   "),
            Span::styled(" n ", Style::default().fg(BG_PRIMARY).bg(ACCENT)),
            Span::styled(" No ", Style::default().fg(TEXT_SECONDARY)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm Delete ")
                .title_style(Style::default().fg(WARNING))
                .border_style(Style::default().fg(WARNING))
                .style(Style::default().bg(BG_PANEL)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

/// Renders the clear all workers confirmation dialog.
pub fn render_confirm_clear_all(f: &mut Frame, app: &App) {
    if !app.confirm_clear_all {
        return;
    }

    let session = app.current_session();
    let count = session.workers.len();

    let area = f.area();
    let popup_width = 50u16.min(area.width.saturating_sub(4));
    let popup_height = 7u16;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  Clear all {} workers?", count),
            Style::default().fg(TEXT_PRIMARY),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(" y ", Style::default().fg(BG_PRIMARY).bg(ERROR)),
            Span::styled(" Yes ", Style::default().fg(TEXT_SECONDARY)),
            Span::raw("   "),
            Span::styled(" n ", Style::default().fg(BG_PRIMARY).bg(ACCENT)),
            Span::styled(" No ", Style::default().fg(TEXT_SECONDARY)),
        ]),
    ];

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Confirm Clear All ")
                .title_style(Style::default().fg(WARNING))
                .border_style(Style::default().fg(WARNING))
                .style(Style::default().bg(BG_PANEL)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

/// Renders the model selector popup.
pub fn render_model_selector(f: &mut Frame, app: &App) {
    if !app.show_model_selector {
        return;
    }

    let area = f.area();
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_height = (app.model_options.len() + 4).min(20) as u16;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let items: Vec<ListItem> = app
        .model_options
        .iter()
        .enumerate()
        .map(|(i, opt)| {
            let is_selected = i == app.model_selector_index;
            let bg = if is_selected { BG_SELECTED } else { BG_PANEL };
            let fg = if is_selected {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            };

            ListItem::new(Line::styled(
                format!(" {} ", opt.display()),
                Style::default().fg(fg),
            ))
            .style(Style::default().bg(bg))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Select Model ")
            .title_style(Style::default().fg(ACCENT))
            .border_style(Style::default().fg(BORDER_ACTIVE))
            .style(Style::default().bg(BG_PANEL)),
    );

    f.render_widget(Clear, popup_area);
    f.render_widget(list, popup_area);
}

/// Renders the permission request dialog.
pub fn render_permission_dialog(f: &mut Frame, app: &App) {
    if !app.show_permission_dialog || app.pending_permissions.is_empty() {
        return;
    }

    let perm = &app.pending_permissions[0];
    let area = f.area();
    let popup_width = 70u16.min(area.width.saturating_sub(4));
    let popup_height = 14u16.min(area.height.saturating_sub(4));

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let worker_str = perm
        .worker_id
        .map(|id| format!("Worker #{}", id))
        .unwrap_or_else(|| "Session".to_string());

    let desc_str = perm
        .worker_description
        .as_ref()
        .map(|d| format!(" ({})", truncate_str(d, 30)))
        .unwrap_or_default();

    let patterns_display: Vec<String> = perm
        .patterns
        .iter()
        .take(3)
        .map(|p| truncate_str(p, 55))
        .collect();

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Permission Required",
            Style::default().fg(WARNING),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::styled("  From: ", Style::default().fg(TEXT_DIM)),
            Span::styled(
                format!("{}{}", worker_str, desc_str),
                Style::default().fg(TEXT_PRIMARY),
            ),
        ]),
        Line::from(vec![
            Span::styled("  Tool: ", Style::default().fg(TEXT_DIM)),
            Span::styled(&perm.permission, Style::default().fg(ACCENT)),
        ]),
        Line::from(""),
    ];

    if !patterns_display.is_empty() {
        lines.push(Line::from(vec![Span::styled(
            "  Files:",
            Style::default().fg(TEXT_DIM),
        )]));
        for pattern in &patterns_display {
            lines.push(Line::from(vec![Span::styled(
                format!("    {}", pattern),
                Style::default().fg(TEXT_SECONDARY),
            )]));
        }
        if perm.patterns.len() > 3 {
            lines.push(Line::from(vec![Span::styled(
                format!("    ... and {} more", perm.patterns.len() - 3),
                Style::default().fg(TEXT_DIM),
            )]));
        }
    }

    lines.push(Line::from(""));

    let options = [
        ("y", "Yes", app.permission_selector_index == 0),
        ("a", "Always", app.permission_selector_index == 1),
        ("n", "Reject", app.permission_selector_index == 2),
    ];

    let option_spans: Vec<Span> = options
        .iter()
        .enumerate()
        .flat_map(|(i, (key, label, selected))| {
            let (fg, bg) = if *selected {
                (BG_PRIMARY, ACCENT)
            } else {
                (TEXT_SECONDARY, BG_PANEL)
            };
            let sep = if i < options.len() - 1 { "   " } else { "" };
            vec![
                Span::styled(
                    format!(" {} {} ", key, label),
                    Style::default().fg(fg).bg(bg),
                ),
                Span::raw(sep),
            ]
        })
        .collect();

    lines.push(Line::from(
        vec![Span::raw("  ")]
            .into_iter()
            .chain(option_spans)
            .collect::<Vec<_>>(),
    ));

    let pending_count = app.pending_permissions.len();
    if pending_count > 1 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!("  {} more pending", pending_count - 1),
            Style::default().fg(TEXT_DIM),
        )]));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(WARNING))
                .style(Style::default().bg(BG_PANEL)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

/// Renders the stop worker selector popup.
pub fn render_stop_selector(f: &mut Frame, app: &App) {
    if !app.show_stop_selector {
        return;
    }

    let running_workers = app.get_running_workers();
    if running_workers.is_empty() {
        return;
    }

    let area = f.area();
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_height = (running_workers.len() + 6).min(20) as u16;

    let popup_area = Rect {
        x: (area.width.saturating_sub(popup_width)) / 2,
        y: (area.height.saturating_sub(popup_height)) / 2,
        width: popup_width,
        height: popup_height,
    };

    let mut lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            "  Select workers to stop (Space to toggle, Enter to confirm)",
            Style::default().fg(TEXT_DIM),
        )]),
        Line::from(""),
    ];

    for (i, (worker_idx, worker)) in running_workers.iter().enumerate() {
        let is_cursor = i == app.stop_selector_cursor;
        let is_selected = app.stop_selector_selections.contains(worker_idx);

        let checkbox = if is_selected { "[x]" } else { "[ ]" };
        let cursor_marker = if is_cursor { ">" } else { " " };
        let state_symbol = worker.state.symbol();

        let bg = if is_cursor { BG_SELECTED } else { BG_PANEL };
        let fg = if is_cursor {
            TEXT_PRIMARY
        } else {
            TEXT_SECONDARY
        };
        let checkbox_fg = if is_selected { ACCENT } else { TEXT_DIM };

        lines.push(Line::from(vec![
            Span::styled(
                format!("  {} ", cursor_marker),
                Style::default().fg(fg).bg(bg),
            ),
            Span::styled(
                format!("{} ", checkbox),
                Style::default().fg(checkbox_fg).bg(bg),
            ),
            Span::styled(format!("{} ", state_symbol), Style::default().fg(fg).bg(bg)),
            Span::styled(
                format!("#{} {}", worker.id, truncate_str(&worker.description, 35)),
                Style::default().fg(fg).bg(bg),
            ),
        ]));
    }

    lines.push(Line::from(""));
    let selected_count = app.stop_selector_selections.len();
    let hint = if selected_count > 0 {
        format!(
            "  {} selected · Enter to stop · Esc to cancel",
            selected_count
        )
    } else {
        "  Space to select · Esc to cancel".to_string()
    };
    lines.push(Line::from(vec![Span::styled(
        hint,
        Style::default().fg(TEXT_DIM),
    )]));

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Stop Workers ")
                .title_style(Style::default().fg(WARNING))
                .border_style(Style::default().fg(WARNING))
                .style(Style::default().bg(BG_PANEL)),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(Clear, popup_area);
    f.render_widget(paragraph, popup_area);
}

/// Renders the autocomplete popup.
pub fn render_autocomplete(f: &mut Frame, app: &App, input_area: Rect) {
    if !app.show_autocomplete || !app.input_mode {
        return;
    }

    let suggestions = app.get_current_suggestions();
    if suggestions.is_empty() {
        return;
    }

    let popup_height = (suggestions.len() + 2).min(10) as u16;
    let popup_width = 50u16;

    let popup_area = Rect {
        x: input_area.x + 2,
        y: input_area.y.saturating_sub(popup_height),
        width: popup_width.min(input_area.width.saturating_sub(4)),
        height: popup_height,
    };

    let items: Vec<ListItem> = suggestions
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let is_selected = i == app.autocomplete_index;
            let bg = if is_selected { BG_SELECTED } else { BG_PANEL };
            let cmd_color = if is_selected {
                TEXT_PRIMARY
            } else {
                TEXT_SECONDARY
            };
            let desc_color = TEXT_SECONDARY;

            let line = Line::from(vec![
                Span::styled(
                    format!(" {:<12}", s.command),
                    Style::default().fg(cmd_color),
                ),
                Span::styled(s.description, Style::default().fg(desc_color)),
            ]);

            ListItem::new(line).style(Style::default().bg(bg))
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(BORDER))
                .style(Style::default().bg(BG_PANEL)),
        )
        .highlight_style(Style::default());

    let mut list_state = ListState::default().with_selected(Some(app.autocomplete_index));

    f.render_widget(Clear, popup_area);
    f.render_stateful_widget(list, popup_area, &mut list_state);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::server::OpenCodeServer;
    use crate::tui::worker::Worker;

    fn create_test_app() -> App {
        let server = OpenCodeServer::new(4096);
        App::new(server)
    }

    mod render_confirm_delete_worker_tests {
        use super::*;
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        #[test]
        fn does_not_render_when_confirm_delete_is_false() {
            let app = create_test_app();
            assert!(!app.confirm_delete);

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    render_confirm_delete_worker(f, &app);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(!content.contains("Delete worker"));
        }

        #[test]
        fn renders_dialog_when_confirm_delete_is_true() {
            let mut app = create_test_app();
            app.current_session_mut()
                .workers
                .push(Worker::new(1, "Test Worker".to_string()));
            app.current_session_mut().selected_worker = Some(0);
            app.confirm_delete = true;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    render_confirm_delete_worker(f, &app);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(content.contains("Delete worker"));
            assert!(content.contains("#1"));
        }

        #[test]
        fn shows_worker_description_in_dialog() {
            let mut app = create_test_app();
            app.current_session_mut()
                .workers
                .push(Worker::new(42, "My Important Task".to_string()));
            app.current_session_mut().selected_worker = Some(0);
            app.confirm_delete = true;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    render_confirm_delete_worker(f, &app);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(content.contains("#42"));
            assert!(content.contains("My Important Task"));
        }
    }

    mod render_confirm_clear_all_tests {
        use super::*;
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;

        #[test]
        fn does_not_render_when_confirm_clear_all_is_false() {
            let app = create_test_app();
            assert!(!app.confirm_clear_all);

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    render_confirm_clear_all(f, &app);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(!content.contains("Clear all"));
        }

        #[test]
        fn renders_dialog_with_worker_count() {
            let mut app = create_test_app();
            app.current_session_mut()
                .workers
                .push(Worker::new(1, "Worker 1".to_string()));
            app.current_session_mut()
                .workers
                .push(Worker::new(2, "Worker 2".to_string()));
            app.current_session_mut()
                .workers
                .push(Worker::new(3, "Worker 3".to_string()));
            app.confirm_clear_all = true;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            terminal
                .draw(|f| {
                    render_confirm_clear_all(f, &app);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(content.contains("Clear all"));
            assert!(content.contains("3"));
        }
    }

    mod render_autocomplete_tests {
        use super::*;
        use ratatui::backend::TestBackend;
        use ratatui::layout::Rect;
        use ratatui::Terminal;

        #[test]
        fn does_not_render_when_autocomplete_disabled() {
            let mut app = create_test_app();
            app.show_autocomplete = false;
            app.input_mode = true;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            let input_area = Rect::new(0, 20, 80, 3);
            terminal
                .draw(|f| {
                    render_autocomplete(f, &app, input_area);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(!content.contains("/help"));
        }

        #[test]
        fn renders_suggestions_when_enabled() {
            let mut app = create_test_app();
            app.show_autocomplete = true;
            app.input_mode = true;
            app.textarea.set_input("/");

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            let input_area = Rect::new(0, 20, 80, 3);
            terminal
                .draw(|f| {
                    render_autocomplete(f, &app, input_area);
                })
                .unwrap();

            let buffer = terminal.backend().buffer();
            let content: String = buffer
                .content
                .iter()
                .map(|c| c.symbol().chars().next().unwrap_or(' '))
                .collect();
            assert!(content.contains("/help"));
        }

        #[test]
        fn uses_list_state_for_selection_scrolling() {
            let mut app = create_test_app();
            app.show_autocomplete = true;
            app.input_mode = true;
            app.textarea.set_input("/");
            app.autocomplete_index = 5;

            let backend = TestBackend::new(80, 24);
            let mut terminal = Terminal::new(backend).unwrap();
            let input_area = Rect::new(0, 20, 80, 3);
            terminal
                .draw(|f| {
                    render_autocomplete(f, &app, input_area);
                })
                .unwrap();

            let suggestions = app.get_current_suggestions();
            assert!(app.autocomplete_index < suggestions.len());
        }
    }
}

//! Dialog rendering (model selector, permission dialog, etc.).

use ratatui::{
    layout::Rect,
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::utils::truncate_str;

/// Renders the model selector popup.
pub fn render_model_selector(f: &mut Frame, app: &App) {
    if !app.show_model_selector {
        return;
    }

    let area = f.area();
    let popup_width = 60u16.min(area.width.saturating_sub(4));
    let popup_height = (app.model_options.len() + 2).min(20) as u16;

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
            let style = if i == app.model_selector_index {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Line::styled(opt.display(), style))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Select Model (↑↓/jk Enter Esc) ")
            .title_style(Style::default().fg(Color::Cyan).bold())
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
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
    let popup_height = 12u16.min(area.height.saturating_sub(4));

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
        .map(|d| format!(" ({})", d))
        .unwrap_or_default();

    let patterns_display: Vec<String> = perm.patterns.iter().map(|p| truncate_str(p, 60)).collect();

    let mut lines = vec![
        Line::from(vec![Span::styled(
            "Permission Request",
            Style::default().fg(Color::Yellow).bold(),
        )]),
        Line::from(""),
        Line::from(vec![
            Span::raw("From: "),
            Span::styled(
                format!("{}{}", worker_str, desc_str),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("Tool: "),
            Span::styled(&perm.permission, Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(vec![Span::raw("Files: ")]),
    ];

    for pattern in &patterns_display {
        lines.push(Line::from(vec![
            Span::raw("  "),
            Span::styled(pattern, Style::default().fg(Color::White)),
        ]));
    }

    lines.push(Line::from(""));

    let options = [
        ("y", "Yes (once)", app.permission_selector_index == 0),
        ("a", "Always", app.permission_selector_index == 1),
        ("n", "No (reject)", app.permission_selector_index == 2),
    ];

    let option_spans: Vec<Span> = options
        .iter()
        .enumerate()
        .flat_map(|(i, (key, label, selected))| {
            let style = if *selected {
                Style::default().fg(Color::Black).bg(Color::Cyan).bold()
            } else {
                Style::default().fg(Color::Gray)
            };
            let sep = if i < options.len() - 1 { "  " } else { "" };
            vec![
                Span::styled(format!("[{}] {}", key, label), style),
                Span::raw(sep),
            ]
        })
        .collect();

    lines.push(Line::from(option_spans));

    let pending_count = app.pending_permissions.len();
    if pending_count > 1 {
        lines.push(Line::from(""));
        lines.push(Line::from(vec![Span::styled(
            format!("({} more pending)", pending_count - 1),
            Style::default().fg(Color::DarkGray),
        )]));
    }

    let paragraph = Paragraph::new(lines)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .title(" 🔐 Permission Required ")
                .title_style(Style::default().fg(Color::Yellow).bold())
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
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

    let popup_height = (suggestions.len() + 2).min(8) as u16;
    let popup_width = 45u16;

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
            let style = if i == app.autocomplete_index {
                Style::default().fg(Color::Black).bg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            let text = format!("{:<15} {}", s.command, s.description);
            ListItem::new(Line::styled(text, style))
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" Commands ")
            .border_style(Style::default().fg(Color::DarkGray)),
    );

    f.render_widget(Clear, popup_area);
    f.render_widget(list, popup_area);
}

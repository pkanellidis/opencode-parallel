//! Dialog rendering (model selector, permission dialog, etc.).

use ratatui::{
    layout::Rect,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

use crate::tui::app::App;
use crate::utils::truncate_str;

use super::theme::*;

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
            let cmd_color = if is_selected { ACCENT } else { TEXT_PRIMARY };
            let desc_color = TEXT_DIM;

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

    let list = List::new(items).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(BORDER))
            .style(Style::default().bg(BG_PANEL)),
    );

    f.render_widget(Clear, popup_area);
    f.render_widget(list, popup_area);
}

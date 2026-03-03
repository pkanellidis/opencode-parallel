//! TUI module for opencode-parallel.
//!
//! This module provides a terminal user interface for managing parallel workers.

pub mod app;
pub mod commands;
pub mod messages;
pub mod scroll;
pub mod selection;
pub mod session;
pub mod textarea;
pub mod tool_display;
pub mod ui;
pub mod worker;

mod handlers;

pub use app::App;
pub use commands::SlashCommand;
pub use messages::{AppMessage, ModelOption, PendingPermission};
pub use selection::TextSelection;
pub use session::Session;
pub use ui::render::ui;
pub use worker::{Worker, WorkerState};

use anyhow::Result;
use crossterm::{
    event::{
        self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture,
        Event, KeyEventKind, KeyboardEnhancementFlags, PopKeyboardEnhancementFlags,
        PushKeyboardEnhancementFlags,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::Alignment,
    style::{Color, Style},
    widgets::Paragraph,
    Terminal,
};
use std::io;
use tokio::sync::mpsc;

use crate::constants::{CHANNEL_BUFFER_SIZE, DEFAULT_PORT, POLL_TIMEOUT_MS};
use crate::server::{OpenCodeServer, ServerProcess, StreamEvent};

pub async fn run_tui(_num_agents: usize, _workdir: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(
        stdout,
        EnterAlternateScreen,
        EnableMouseCapture,
        EnableBracketedPaste
    )?;

    // Enable enhanced keyboard protocol for proper Shift+Enter detection
    // This uses the Kitty keyboard protocol which is supported by modern terminals
    let supports_keyboard_enhancement =
        crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
    if supports_keyboard_enhancement {
        execute!(
            stdout,
            PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_ALL_KEYS_AS_ESCAPE_CODES)
        )?;
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let area = f.area();
        let block = Paragraph::new("Starting opencode server...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(block, area);
    })?;

    let mut server_process: ServerProcess = ServerProcess::start(DEFAULT_PORT).await?;
    let server = OpenCodeServer::new(DEFAULT_PORT);

    let mut app = App::new(server.clone());

    app.status = "Initializing orchestrator...".to_string();
    terminal.draw(|f| ui(f, &mut app))?;

    app.orchestrator.init().await?;
    app.status = "Ready - Type your task".to_string();
    terminal.draw(|f| ui(f, &mut app))?;

    let (tx, mut rx) = mpsc::channel::<AppMessage>(CHANNEL_BUFFER_SIZE);

    let server_clone = server.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        if let Ok(config) = server_clone.get_config().await {
            let model = config
                .get("model")
                .and_then(|m| m.as_str())
                .map(String::from);
            let _ = tx_clone.send(AppMessage::CurrentModelLoaded(model)).await;
        }
    });

    let (sse_tx, mut sse_rx) = mpsc::channel::<StreamEvent>(CHANNEL_BUFFER_SIZE);
    server.subscribe_events(sse_tx);

    let tx_sse = tx.clone();
    tokio::spawn(async move {
        while let Some(event) = sse_rx.recv().await {
            if tx_sse.send(AppMessage::StreamEvent(event)).await.is_err() {
                break;
            }
        }
    });

    loop {
        // Update scroll state for momentum scrolling
        app.tick_scroll();

        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(POLL_TIMEOUT_MS))? {
            match event::read()? {
                Event::Mouse(mouse) => {
                    let _ = handlers::handle_mouse_event(&mut app, mouse);
                }
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handlers::handle_key_event(&mut app, key, &server, &tx).await;
                }
                Event::Paste(text) => {
                    handlers::handle_paste_event(&mut app, text);
                }
                _ => {}
            }
        }

        while let Ok(msg) = rx.try_recv() {
            handlers::handle_app_message(&mut app, msg, &server, &tx).await;
        }

        if app.quit {
            break;
        }
    }

    server_process.stop().await?;

    // Pop keyboard enhancement flags if we pushed them
    let supports_keyboard_enhancement =
        crossterm::terminal::supports_keyboard_enhancement().unwrap_or(false);
    if supports_keyboard_enhancement {
        execute!(terminal.backend_mut(), PopKeyboardEnhancementFlags)?;
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture,
        DisableBracketedPaste
    )?;
    terminal.show_cursor()?;

    Ok(())
}

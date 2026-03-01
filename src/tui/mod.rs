//! TUI module for opencode-parallel.
//!
//! This module provides a terminal user interface for managing parallel workers.

pub mod app;
pub mod commands;
pub mod messages;
pub mod selection;
pub mod session;
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
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind},
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

use crate::server::{OpenCodeServer, ServerProcess, StreamEvent};

pub async fn run_tui(_num_agents: usize, _workdir: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let area = f.area();
        let block = Paragraph::new("Starting opencode server...")
            .style(Style::default().fg(Color::Yellow))
            .alignment(Alignment::Center);
        f.render_widget(block, area);
    })?;

    let port = 14096u16;
    let mut server_process: ServerProcess = ServerProcess::start(port).await?;
    let server = OpenCodeServer::new(port);

    let mut app = App::new(server.clone());

    app.status = "Initializing orchestrator...".to_string();
    terminal.draw(|f| ui(f, &mut app))?;

    app.orchestrator.init().await?;
    app.status = "Ready - Type your task".to_string();
    terminal.draw(|f| ui(f, &mut app))?;

    let (tx, mut rx) = mpsc::channel::<AppMessage>(100);

    // Fetch current model on startup
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

    let (sse_tx, mut sse_rx) = mpsc::channel::<StreamEvent>(100);
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
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Mouse(mouse) => {
                    handlers::handle_mouse_event(&mut app, mouse);
                }
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    handlers::handle_key_event(&mut app, key, &server, &tx).await;
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
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

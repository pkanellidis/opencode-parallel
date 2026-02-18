use anyhow::Result;
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, BorderType, Gauge, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use std::io;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::agent::{AgentConfig, AgentStatus};

enum AppMode {
    Normal,
    Writing(String), // The message being typed
}

struct App {
    agents: Vec<AgentConfig>,
    selected: usize,
    scroll_offset: usize,
    quit: bool,
    mode: AppMode,
}

impl App {
    fn new(num_agents: usize) -> Self {
        let agents = (0..num_agents)
            .map(|i| {
                AgentConfig::new(
                    "anthropic",
                    "claude-3-5-sonnet-20241022",
                    &format!("Task {}", i + 1),
                )
            })
            .collect();

        Self {
            agents,
            selected: 0,
            scroll_offset: 0,
            quit: false,
            mode: AppMode::Normal,
        }
    }

    fn next(&mut self) {
        if self.selected < self.agents.len() - 1 {
            self.selected += 1;
            self.scroll_offset = 0;
        }
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            self.scroll_offset = 0;
        }
    }

    fn start_selected(&mut self) {
        if let Some(agent) = self.agents.get_mut(self.selected) {
            if agent.status == AgentStatus::Pending || agent.status == AgentStatus::Cancelled {
                agent.start();
            }
        }
    }

    fn cancel_selected(&mut self) {
        if let Some(agent) = self.agents.get_mut(self.selected) {
            if agent.status == AgentStatus::Running {
                agent.cancel();
            }
        }
    }

    fn scroll_down(&mut self) {
        if let Some(agent) = self.agents.get(self.selected) {
            if self.scroll_offset < agent.output.len().saturating_sub(10) {
                self.scroll_offset += 1;
            }
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn enter_write_mode(&mut self) {
        if let Some(agent) = self.agents.get(self.selected) {
            if agent.status == AgentStatus::Running {
                self.mode = AppMode::Writing(String::new());
            }
        }
    }

    fn exit_write_mode(&mut self) {
        self.mode = AppMode::Normal;
    }

    fn get_progress(&self, index: usize) -> f64 {
        if let Some(agent) = self.agents.get(index) {
            match agent.status {
                AgentStatus::Pending => 0.0,
                AgentStatus::Running => {
                    // Estimate progress based on output lines (max 100 lines = 100%)
                    (agent.output.len() as f64 / 100.0).min(0.95)
                }
                AgentStatus::Completed => 1.0,
                AgentStatus::Failed => 0.0,
                AgentStatus::Cancelled => 0.0,
            }
        } else {
            0.0
        }
    }
}

pub async fn run_tui(num_agents: usize, _workdir: &str) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(num_agents);

    let (tx, mut rx) = mpsc::channel(100);

    let mut task_set = JoinSet::new();
    for (idx, agent) in app.agents.iter().cloned().enumerate() {
        let tx = tx.clone();
        task_set.spawn(async move {
            (idx, simulate_agent_work(agent, tx).await)
        });
    }

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match &mut app.mode {
                        AppMode::Normal => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            KeyCode::Char('s') => app.start_selected(),
                            KeyCode::Char('c') => app.cancel_selected(),
                            KeyCode::Char('d') => app.scroll_down(),
                            KeyCode::Char('u') => app.scroll_up(),
                            KeyCode::Char('w') => app.enter_write_mode(),
                            _ => {}
                        },
                        AppMode::Writing(input) => match key.code {
                            KeyCode::Esc => app.exit_write_mode(),
                            KeyCode::Enter => {
                                let message = input.clone();
                                // TODO: Send message to selected agent's stdin
                                if let Some(agent) = app.agents.get_mut(app.selected) {
                                    agent.add_output(format!("→ You: {}", message));
                                }
                                app.exit_write_mode();
                            }
                            KeyCode::Backspace => {
                                input.pop();
                            }
                            KeyCode::Char(c) => {
                                input.push(c);
                            }
                            _ => {}
                        },
                    }
                }
            }
        }

        while let Ok((idx, updated_agent)) = rx.try_recv() {
            if let Some(agent) = app.agents.get_mut(idx) {
                *agent = updated_agent;
            }
        }

        if app.quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(0),          // Main content
            Constraint::Length(3),       // Footer/Status
        ])
        .split(f.area());

    render_header(f, chunks[0]);

    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(35), Constraint::Percentage(65)])
        .split(chunks[1]);

    render_agent_list(f, app, main_chunks[0]);
    render_agent_detail(f, app, main_chunks[1]);

    render_footer(f, app, chunks[2]);
}

fn render_header(f: &mut Frame, area: Rect) {
    let title = Paragraph::new("╔═══════════════════════════════════════════════════════════╗\n║  OPENCODE PARALLEL - Multiple AI Agents in Parallel  ║\n╚═══════════════════════════════════════════════════════════╝")
        .style(Style::default().fg(Color::Cyan).bold())
        .alignment(Alignment::Center);
    f.render_widget(title, area);
}

fn render_agent_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let (status_icon, status_color) = match agent.status {
                AgentStatus::Pending => ("⏸", Color::Gray),
                AgentStatus::Running => ("▶", Color::Yellow),
                AgentStatus::Completed => ("✓", Color::Green),
                AgentStatus::Failed => ("✗", Color::Red),
                AgentStatus::Cancelled => ("⊘", Color::DarkGray),
            };

            let progress = app.get_progress(i);
            let progress_bar = if progress > 0.0 && progress < 1.0 {
                let filled = (progress * 20.0) as usize;
                let empty = 20 - filled;
                format!(" [{}{}]", "█".repeat(filled), "░".repeat(empty))
            } else {
                String::new()
            };

            let duration = if let Some(d) = agent.duration() {
                format!(" ({}s)", d.num_seconds())
            } else if agent.started_at.is_some() {
                let elapsed = chrono::Utc::now() - agent.started_at.unwrap();
                format!(" ({}s)", elapsed.num_seconds())
            } else {
                String::new()
            };

            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Cyan)
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", status_icon), Style::default().fg(status_color).bold()),
                Span::styled(
                    format!("Agent {}{}{}", i + 1, duration, progress_bar),
                    style,
                ),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .title(" 🤖 Agents ")
                .title_style(Style::default().fg(Color::Cyan).bold())
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(list, area);
}

fn render_agent_detail(f: &mut Frame, app: &App, area: Rect) {
    if let Some(agent) = app.agents.get(app.selected) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Progress bar
                Constraint::Min(0),      // Output
            ])
            .split(area);

        // Progress gauge
        let progress = app.get_progress(app.selected);
        let gauge_color = match agent.status {
            AgentStatus::Running => Color::Yellow,
            AgentStatus::Completed => Color::Green,
            AgentStatus::Failed => Color::Red,
            _ => Color::Gray,
        };

        let gauge = Gauge::default()
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!(" 📊 Agent {} - {} ", app.selected + 1, agent.task))
                    .title_style(Style::default().fg(Color::Cyan).bold())
            )
            .gauge_style(Style::default().fg(gauge_color).bg(Color::Black))
            .percent((progress * 100.0) as u16)
            .label(format!("{:.0}%", progress * 100.0));

        f.render_widget(gauge, chunks[0]);

        // Output
        let output_height = chunks[1].height.saturating_sub(2) as usize;
        let output_lines: Vec<Line> = agent
            .output
            .iter()
            .skip(app.scroll_offset)
            .take(output_height)
            .map(|line| {
                if line.starts_with("→ You:") {
                    Line::from(vec![
                        Span::styled("→ ", Style::default().fg(Color::Magenta).bold()),
                        Span::styled(&line[6..], Style::default().fg(Color::Magenta)),
                    ])
                } else if line.starts_with("✓") {
                    Line::styled(line.as_str(), Style::default().fg(Color::Green))
                } else if line.starts_with("✗") {
                    Line::styled(line.as_str(), Style::default().fg(Color::Red))
                } else {
                    Line::from(line.as_str())
                }
            })
            .collect();

        let output = Paragraph::new(output_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(format!(" 💬 Output (scroll: {}/{}) ", 
                        app.scroll_offset, 
                        agent.output.len().saturating_sub(output_height)))
                    .title_style(Style::default().fg(Color::Yellow).bold())
                    .border_style(Style::default().fg(Color::Yellow)),
            )
            .wrap(Wrap { trim: false });

        f.render_widget(output, chunks[1]);
    }
}

fn render_footer(f: &mut Frame, app: &App, area: Rect) {
    let text = match &app.mode {
        AppMode::Normal => {
            vec![
                Span::styled(" q:", Style::default().fg(Color::Red).bold()),
                Span::raw("Quit "),
                Span::styled("↑/k:", Style::default().fg(Color::Cyan).bold()),
                Span::raw("Up "),
                Span::styled("↓/j:", Style::default().fg(Color::Cyan).bold()),
                Span::raw("Down "),
                Span::styled("s:", Style::default().fg(Color::Green).bold()),
                Span::raw("Start "),
                Span::styled("c:", Style::default().fg(Color::Yellow).bold()),
                Span::raw("Cancel "),
                Span::styled("w:", Style::default().fg(Color::Magenta).bold()),
                Span::raw("Write "),
                Span::styled("u/d:", Style::default().fg(Color::Blue).bold()),
                Span::raw("Scroll "),
            ]
        }
        AppMode::Writing(input) => {
            vec![
                Span::styled(" 💬 Message: ", Style::default().fg(Color::Magenta).bold()),
                Span::raw(input.as_str()),
                Span::styled("█", Style::default().fg(Color::Cyan)),
                Span::styled("  [Enter]", Style::default().fg(Color::Green).bold()),
                Span::raw(" Send "),
                Span::styled("[Esc]", Style::default().fg(Color::Red).bold()),
                Span::raw(" Cancel "),
            ]
        }
    };

    let help = Paragraph::new(Line::from(text))
        .style(Style::default().bg(Color::DarkGray))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::Gray)),
        );

    f.render_widget(help, area);
}

async fn simulate_agent_work(mut agent: AgentConfig, tx: mpsc::Sender<(usize, AgentConfig)>) {
    agent.start();
    agent.add_output(format!("🚀 Starting opencode for: {}", agent.task));
    agent.add_output(format!("📦 Model: {} / {}", agent.provider, agent.model));
    let _ = tx.send((0, agent.clone())).await;

    // Build opencode command
    let mut cmd = Command::new("opencode");
    cmd.arg("run");
    cmd.arg(&agent.task);
    
    // Add model flag if specified
    if !agent.model.is_empty() {
        cmd.arg("--model");
        cmd.arg(format!("{}/{}", agent.provider, agent.model));
    }
    
    // Capture stdout and stderr
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.stdin(Stdio::piped());
    
    // Spawn the process
    match cmd.spawn() {
        Ok(mut child) => {
            // Get stdout
            if let Some(stdout) = child.stdout.take() {
                let mut stdout_reader = BufReader::new(stdout).lines();
                
                // Read stdout line by line
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    agent.add_output(line);
                    let _ = tx.send((0, agent.clone())).await;
                }
            }
            
            // Wait for process to complete
            match child.wait().await {
                Ok(status) => {
                    if status.success() {
                        agent.add_output("✓ Task completed successfully!".to_string());
                        agent.complete();
                    } else {
                        agent.add_output(format!("✗ Failed with exit code: {:?}", status.code()));
                        agent.fail();
                    }
                }
                Err(e) => {
                    agent.add_output(format!("✗ Error waiting for process: {}", e));
                    agent.fail();
                }
            }
        }
        Err(e) => {
            agent.add_output(format!("✗ Failed to spawn opencode: {}", e));
            agent.fail();
        }
    }
    
    let _ = tx.send((0, agent)).await;
}

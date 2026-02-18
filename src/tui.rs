use anyhow::{Context, Result};
use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::agent::{AgentConfig, AgentStatus};

struct App {
    agents: Vec<AgentConfig>,
    selected: usize,
    scroll_offset: usize,
    quit: bool,
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
        }
    }

    fn next(&mut self) {
        if self.selected < self.agents.len() - 1 {
            self.selected += 1;
        }
    }

    fn previous(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
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
    for agent in app.agents.iter().cloned() {
        let tx = tx.clone();
        task_set.spawn(async move {
            simulate_agent_work(agent, tx).await
        });
    }

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => app.quit = true,
                        KeyCode::Down | KeyCode::Char('j') => app.next(),
                        KeyCode::Up | KeyCode::Char('k') => app.previous(),
                        KeyCode::Char('s') => app.start_selected(),
                        KeyCode::Char('c') => app.cancel_selected(),
                        KeyCode::Char('d') => app.scroll_down(),
                        KeyCode::Char('u') => app.scroll_up(),
                        _ => {}
                    }
                }
            }
        }

        while let Ok(update) = rx.try_recv() {
            if let Some(agent) = app.agents.iter_mut().find(|a| a.id == update.id) {
                agent.status = update.status;
                agent.output = update.output;
                agent.started_at = update.started_at;
                agent.completed_at = update.completed_at;
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
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(40), Constraint::Percentage(60)].as_ref())
        .split(f.area());

    render_agent_list(f, app, chunks[0]);
    render_agent_detail(f, app, chunks[1]);

    render_help(f, f.area());
}

fn render_agent_list(f: &mut Frame, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .agents
        .iter()
        .enumerate()
        .map(|(i, agent)| {
            let status_char = match agent.status {
                AgentStatus::Pending => "○",
                AgentStatus::Running => "◉",
                AgentStatus::Completed => "✓",
                AgentStatus::Failed => "✗",
                AgentStatus::Cancelled => "⊘",
            };

            let status_color = match agent.status {
                AgentStatus::Pending => Color::Gray,
                AgentStatus::Running => Color::Yellow,
                AgentStatus::Completed => Color::Green,
                AgentStatus::Failed => Color::Red,
                AgentStatus::Cancelled => Color::DarkGray,
            };

            let style = if i == app.selected {
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            let duration = if let Some(d) = agent.duration() {
                format!(" ({}s)", d.num_seconds())
            } else if agent.started_at.is_some() {
                let elapsed = chrono::Utc::now() - agent.started_at.unwrap();
                format!(" ({}s)", elapsed.num_seconds())
            } else {
                String::new()
            };

            let line = Line::from(vec![
                Span::styled(format!("{} ", status_char), Style::default().fg(status_color)),
                Span::styled(
                    format!("Agent {} {}", i + 1, duration),
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
                .title(" Agents ")
                .border_style(Style::default().fg(Color::Cyan)),
        );

    f.render_widget(list, area);
}

fn render_agent_detail(f: &mut Frame, app: &App, area: Rect) {
    if let Some(agent) = app.agents.get(app.selected) {
        let output_lines: Vec<Line> = agent
            .output
            .iter()
            .skip(app.scroll_offset)
            .take(area.height.saturating_sub(2) as usize)
            .map(|line| Line::from(line.as_str()))
            .collect();

        let detail = Paragraph::new(output_lines)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(format!(" Agent {} - {} ", app.selected + 1, agent.task))
                    .border_style(Style::default().fg(Color::Cyan)),
            );

        f.render_widget(detail, area);
    }
}

fn render_help(f: &mut Frame, area: Rect) {
    let help_text = " q:Quit | ↑/k:Up | ↓/j:Down | s:Start | c:Cancel | u/d:Scroll ";
    let help = Paragraph::new(help_text)
        .style(Style::default().fg(Color::DarkGray))
        .block(Block::default());

    let help_area = Rect {
        x: area.x,
        y: area.height.saturating_sub(1),
        width: area.width,
        height: 1,
    };

    f.render_widget(help, help_area);
}

async fn simulate_agent_work(mut agent: AgentConfig, tx: mpsc::Sender<AgentConfig>) {
    agent.start();
    agent.add_output(format!("Starting opencode for: {}", agent.task));
    agent.add_output(format!("Model: {} / {}", agent.provider, agent.model));
    let _ = tx.send(agent.clone()).await;

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
    
    // Spawn the process
    match cmd.spawn() {
        Ok(mut child) => {
            // Get stdout
            if let Some(stdout) = child.stdout.take() {
                let mut stdout_reader = BufReader::new(stdout).lines();
                
                // Read stdout line by line
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    agent.add_output(line);
                    let _ = tx.send(agent.clone()).await;
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
    
    let _ = tx.send(agent).await;
}

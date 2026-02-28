use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;
use tokio::task::JoinSet;

use crate::agent::{AgentConfig, AgentStatus};

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskConfig {
    pub tasks: Vec<TaskDefinition>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDefinition {
    pub provider: String,
    pub model: String,
    pub task: String,
}

pub async fn run_batch(config_path: &str, max_parallel: usize) -> Result<()> {
    let content = fs::read_to_string(config_path)?;
    let config: TaskConfig = serde_json::from_str(&content)?;

    println!(
        "Running {} tasks with max {} parallel agents",
        config.tasks.len(),
        max_parallel
    );

    let (tx, mut rx) = mpsc::channel(100);
    let mut set = JoinSet::new();
    let mut active = 0;
    let mut completed = 0;
    let total = config.tasks.len();

    let mut task_iter = config.tasks.into_iter();

    while completed < total {
        while active < max_parallel {
            if let Some(task_def) = task_iter.next() {
                let agent = AgentConfig::new(&task_def.provider, &task_def.model, &task_def.task);
                let tx = tx.clone();
                let agent_id = agent.id.clone();

                set.spawn(async move { run_agent(agent, tx).await });

                active += 1;
                println!("[{}] Started: {}", agent_id, task_def.task);
            } else {
                break;
            }
        }

        tokio::select! {
            Some(result) = set.join_next() => {
                match result {
                    Ok(Ok(agent)) => {
                        let status_str = match agent.status {
                            AgentStatus::Completed => "✓",
                            AgentStatus::Failed => "✗",
                            AgentStatus::Cancelled => "⊘",
                            _ => "?",
                        };
                        println!("[{}] {} {}", agent.id, status_str, agent.task);
                    }
                    Ok(Err(e)) => {
                        eprintln!("Agent error: {}", e);
                    }
                    Err(e) => {
                        eprintln!("Task join error: {}", e);
                    }
                }
                active -= 1;
                completed += 1;
            }
            Some(msg) = rx.recv() => {
                println!("  {}", msg);
            }
        }
    }

    println!("\nAll tasks completed!");
    Ok(())
}

async fn run_agent(mut agent: AgentConfig, tx: mpsc::Sender<String>) -> Result<AgentConfig> {
    agent.start();

    let _ = tx
        .send(format!(
            "[{}] Starting opencode for task: {}",
            agent.id, agent.task
        ))
        .await;

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

    let _ = tx
        .send(format!(
            "[{}] Executing: opencode run \"{}\" --model {}/{}",
            agent.id, agent.task, agent.provider, agent.model
        ))
        .await;

    // Spawn the process
    let mut child = cmd.spawn().context("Failed to spawn opencode process")?;

    // Get stdout
    let stdout = child.stdout.take().context("Failed to capture stdout")?;
    let mut stdout_reader = BufReader::new(stdout).lines();

    // Read stdout line by line
    let agent_id = agent.id.clone();
    let tx_stdout = tx.clone();
    let stdout_task = tokio::spawn(async move {
        let mut output_lines = Vec::new();
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            let _ = tx_stdout.send(format!("[{}] {}", agent_id, line)).await;
            output_lines.push(line);
        }
        output_lines
    });

    // Wait for process to complete
    let status = child
        .wait()
        .await
        .context("Failed to wait for opencode process")?;

    // Get all output
    let output_lines = stdout_task.await.context("Failed to read stdout")?;

    // Add output to agent
    for line in output_lines {
        agent.add_output(line);
    }

    // Update agent status based on exit code
    if status.success() {
        agent.complete();
        let _ = tx
            .send(format!("[{}] ✓ Completed successfully", agent.id))
            .await;
    } else {
        agent.fail();
        let _ = tx
            .send(format!(
                "[{}] ✗ Failed with exit code: {:?}",
                agent.id,
                status.code()
            ))
            .await;
    }

    Ok(agent)
}

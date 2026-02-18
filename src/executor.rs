use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
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

    println!("Running {} tasks with max {} parallel agents", config.tasks.len(), max_parallel);
    
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
                
                set.spawn(async move {
                    run_agent(agent, tx).await
                });
                
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
    
    let _ = tx.send(format!("[{}] Starting agent for task: {}", agent.id, agent.task)).await;
    
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    
    agent.add_output("Simulated agent execution...".to_string());
    agent.add_output(format!("Task: {}", agent.task));
    agent.add_output(format!("Provider: {}", agent.provider));
    agent.add_output(format!("Model: {}", agent.model));
    
    let _ = tx.send(format!("[{}] Agent processing...", agent.id)).await;
    
    tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
    
    agent.complete();
    
    let _ = tx.send(format!("[{}] Agent completed", agent.id)).await;
    
    Ok(agent)
}

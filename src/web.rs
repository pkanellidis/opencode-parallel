//! Web server launcher for running the backend and web client together.

use anyhow::Result;
use std::process::Stdio;
use tokio::process::{Child, Command};
use tokio::signal;
use tokio::time::{sleep, Duration};

use crate::constants::{DEFAULT_PORT, HEALTH_CHECK_DELAY_MS, HEALTH_CHECK_MAX_ITERATIONS};
use crate::server::OpenCodeServer;

const WEB_CLIENT_DIR: &str = "web-client";
const VITE_DEV_PORT: u16 = 3000;

pub async fn run_web(port: u16, workdir: &str) -> Result<()> {
    println!("Starting opencode-parallel web interface...\n");

    let original_dir = std::env::current_dir()?;
    if workdir != "." {
        std::env::set_current_dir(workdir)?;
    }
    let working_dir = std::env::current_dir()?;

    println!("  Working directory: {}", working_dir.display());
    println!("  Backend port: {}", port);
    println!("  Frontend port: {}\n", VITE_DEV_PORT);

    let mut backend = start_backend(port).await?;
    println!("  [✓] Backend server started on http://127.0.0.1:{}", port);

    let web_client_path = original_dir.join(WEB_CLIENT_DIR);
    if !web_client_path.exists() {
        backend.kill().await?;
        anyhow::bail!(
            "Web client directory not found: {}\nRun from the opencode-parallel repository root.",
            web_client_path.display()
        );
    }

    let mut frontend = start_frontend(&web_client_path, port).await?;
    println!(
        "  [✓] Frontend dev server started on http://127.0.0.1:{}\n",
        VITE_DEV_PORT
    );

    println!("Open http://127.0.0.1:{} in your browser", VITE_DEV_PORT);
    println!("Press Ctrl+C to stop both servers\n");

    signal::ctrl_c().await?;

    println!("\nShutting down...");
    let _ = frontend.kill().await;
    let _ = backend.kill().await;
    println!("Servers stopped.");

    Ok(())
}

async fn start_backend(port: u16) -> Result<Child> {
    let child = Command::new("opencode")
        .arg("serve")
        .arg("--port")
        .arg(port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    let server = OpenCodeServer::new(port);
    for _ in 0..HEALTH_CHECK_MAX_ITERATIONS {
        sleep(Duration::from_millis(HEALTH_CHECK_DELAY_MS)).await;
        if server.is_healthy().await {
            return Ok(child);
        }
    }

    anyhow::bail!("Backend server failed to start within timeout")
}

async fn start_frontend(web_client_path: &std::path::Path, backend_port: u16) -> Result<Child> {
    let node_modules = web_client_path.join("node_modules");
    if !node_modules.exists() {
        println!("  Installing web client dependencies...");
        let status = Command::new("npm")
            .arg("install")
            .current_dir(web_client_path)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .await?;

        if !status.success() {
            anyhow::bail!("Failed to install web client dependencies");
        }
        println!("  [✓] Dependencies installed");
    }

    let child = Command::new("npm")
        .arg("run")
        .arg("dev")
        .current_dir(web_client_path)
        .env("VITE_BACKEND_PORT", backend_port.to_string())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;

    sleep(Duration::from_secs(2)).await;

    Ok(child)
}

pub fn default_port() -> u16 {
    DEFAULT_PORT
}

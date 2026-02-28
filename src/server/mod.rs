//! OpenCode server API client module.
//!
//! This module provides a client for interacting with the OpenCode server,
//! including session management, messaging, and real-time event streaming.
//!
//! # Example
//!
//! ```no_run
//! use opencode_parallel::server::{OpenCodeServer, StreamEvent};
//! use tokio::sync::mpsc;
//!
//! #[tokio::main]
//! async fn main() {
//!     let server = OpenCodeServer::new(4096);
//!     
//!     // Check server health
//!     if server.is_healthy().await {
//!         // Create a session
//!         let session = server.create_session(Some("My Session")).await.unwrap();
//!         
//!         // Send a message
//!         let response = server.send_message(&session.id, "Hello!").await.unwrap();
//!     }
//! }
//! ```

mod client;
mod events;
mod logs;
mod types;

pub use client::{OpenCodeServer, ServerProcess};
pub use events::StreamEvent;
pub use logs::ServerLogs;
pub use types::*;

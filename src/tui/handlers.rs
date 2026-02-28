//! Event handlers for the TUI.

use crossterm::event::{KeyCode, KeyEvent};
use tokio::sync::mpsc::Sender;

use crate::server::OpenCodeServer;

use super::app::App;
use super::messages::AppMessage;

pub async fn handle_key_event(
    _app: &mut App,
    _key: KeyEvent,
    _server: &OpenCodeServer,
    _tx: &Sender<AppMessage>,
) {
    // TODO: Implement key handling
}

pub async fn handle_app_message(
    _app: &mut App,
    _msg: AppMessage,
    _server: &OpenCodeServer,
    _tx: &Sender<AppMessage>,
) {
    // TODO: Implement message handling
}

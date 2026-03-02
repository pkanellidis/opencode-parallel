//! Application-wide constants.

/// Default port for the OpenCode server.
pub const DEFAULT_PORT: u16 = 14096;

/// Channel buffer size for async message passing.
pub const CHANNEL_BUFFER_SIZE: usize = 100;

/// Poll timeout in milliseconds for the event loop.
pub const POLL_TIMEOUT_MS: u64 = 50;

/// Width of the workers sidebar in characters.
pub const SIDEBAR_WIDTH: u16 = 30;

/// Minimum width of the worker detail panel in characters.
pub const DETAIL_PANEL_MIN_WIDTH: u16 = 40;

/// Ratio of detail panel to main content (percentage of remaining width).
pub const DETAIL_PANEL_RATIO: u16 = 45;

/// Number of lines to scroll when paging.
pub const PAGE_SCROLL_LINES: usize = 20;

/// Number of lines to scroll with arrow keys.
pub const LINE_SCROLL_AMOUNT: usize = 10;

/// Minimum height of the input area.
pub const MIN_INPUT_HEIGHT: u16 = 4;

/// Maximum height of the input area.
pub const MAX_INPUT_HEIGHT: u16 = 12;

/// Maximum characters for task descriptions.
pub const MAX_DESCRIPTION_CHARS: usize = 50;

/// Number of lines to show in worker summary.
pub const SUMMARY_LINE_LIMIT: usize = 10;

/// Health check maximum iterations.
pub const HEALTH_CHECK_MAX_ITERATIONS: u32 = 50;

/// Health check delay in milliseconds.
pub const HEALTH_CHECK_DELAY_MS: u64 = 100;

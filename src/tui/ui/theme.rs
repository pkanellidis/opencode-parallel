//! Theme constants for the TUI.

use ratatui::style::Color;

/// Pure black background
pub const BG_PRIMARY: Color = Color::Rgb(0, 0, 0);

/// Lighter dark panel background (dark charcoal)
pub const BG_PANEL: Color = Color::Rgb(22, 22, 26);

/// Even lighter for hover/selected states
pub const BG_SELECTED: Color = Color::Rgb(38, 38, 44);

/// Border color for panels
pub const BORDER: Color = Color::Rgb(60, 60, 70);

/// Border color when focused/active
pub const BORDER_ACTIVE: Color = Color::Rgb(100, 100, 120);

/// Primary accent color (blue-ish)
pub const ACCENT: Color = Color::Rgb(99, 140, 255);

/// Secondary accent (purple-ish)
pub const ACCENT_SECONDARY: Color = Color::Rgb(180, 130, 255);

/// Success color (green)
pub const SUCCESS: Color = Color::Rgb(80, 200, 120);

/// Warning color (yellow/orange)
pub const WARNING: Color = Color::Rgb(255, 180, 80);

/// Error color (red)
pub const ERROR: Color = Color::Rgb(255, 100, 100);

/// Primary text color
pub const TEXT_PRIMARY: Color = Color::Rgb(230, 230, 235);

/// Secondary/muted text
pub const TEXT_SECONDARY: Color = Color::Rgb(140, 140, 150);

/// Dimmed text
pub const TEXT_DIM: Color = Color::Rgb(90, 90, 100);

/// Running indicator color
pub const STATUS_RUNNING: Color = Color::Rgb(100, 180, 255);

/// Waiting indicator color
pub const STATUS_WAITING: Color = Color::Rgb(255, 200, 100);

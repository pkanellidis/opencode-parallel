//! Scroll acceleration and momentum handling for smooth trackpad scrolling.
//!
//! This module implements macOS-style scroll acceleration similar to the
//! original opencode's @opentui/core MacOSScrollAccel.

use std::time::{Duration, Instant};

/// Trait for scroll acceleration implementations.
pub trait ScrollAcceleration {
    /// Returns the number of lines to scroll based on the current velocity.
    fn tick(&mut self) -> usize;

    /// Called when a scroll event is received.
    fn scroll_event(&mut self, direction: ScrollDirection);

    /// Resets the acceleration state.
    fn reset(&mut self);

    /// Returns true if there's momentum scrolling in progress.
    fn has_momentum(&self) -> bool;
}

/// Direction of scrolling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScrollDirection {
    Up,
    Down,
    Left,
    Right,
}

/// Fixed-speed scroll acceleration (no momentum).
pub struct FixedSpeedScroll {
    speed: usize,
    pending: usize,
    direction: Option<ScrollDirection>,
}

impl FixedSpeedScroll {
    pub fn new(speed: usize) -> Self {
        Self {
            speed,
            pending: 0,
            direction: None,
        }
    }
}

impl ScrollAcceleration for FixedSpeedScroll {
    fn tick(&mut self) -> usize {
        let result = self.pending;
        self.pending = 0;
        result
    }

    fn scroll_event(&mut self, direction: ScrollDirection) {
        self.direction = Some(direction);
        self.pending = self.speed;
    }

    fn reset(&mut self) {
        self.pending = 0;
        self.direction = None;
    }

    fn has_momentum(&self) -> bool {
        false
    }
}

/// macOS-style scroll acceleration with momentum.
///
/// This implements inertial scrolling similar to Apple's trackpad behavior:
/// - Faster scroll movements result in more lines scrolled
/// - Velocity is tracked over recent events
/// - Momentum continues after scrolling stops (optional)
pub struct MacOSScrollAccel {
    /// Recent scroll events for velocity calculation
    events: Vec<(Instant, ScrollDirection)>,
    /// Current velocity (lines per second)
    velocity: f64,
    /// Last tick time for momentum calculation
    last_tick: Instant,
    /// Whether momentum scrolling is active
    momentum_active: bool,
    /// Configuration
    config: ScrollConfig,
}

/// Configuration for scroll acceleration.
pub struct ScrollConfig {
    /// Base scroll amount per event
    pub base_amount: f64,
    /// Maximum velocity multiplier
    pub max_velocity: f64,
    /// Velocity decay factor per tick (0.0-1.0)
    pub decay: f64,
    /// Time window for velocity calculation (ms)
    pub velocity_window_ms: u64,
    /// Minimum velocity to continue momentum
    pub momentum_threshold: f64,
    /// Enable momentum scrolling
    pub momentum_enabled: bool,
}

impl Default for ScrollConfig {
    fn default() -> Self {
        Self {
            base_amount: 1.0,
            max_velocity: 20.0,
            decay: 0.85,
            velocity_window_ms: 100,
            momentum_threshold: 0.5,
            momentum_enabled: true,
        }
    }
}

impl MacOSScrollAccel {
    pub fn new() -> Self {
        Self::with_config(ScrollConfig::default())
    }

    pub fn with_config(config: ScrollConfig) -> Self {
        Self {
            events: Vec::new(),
            velocity: 0.0,
            last_tick: Instant::now(),
            momentum_active: false,
            config,
        }
    }

    /// Calculate velocity based on recent events.
    fn calculate_velocity(&mut self) {
        let now = Instant::now();
        let window = Duration::from_millis(self.config.velocity_window_ms);

        // Remove old events
        self.events
            .retain(|(time, _)| now.duration_since(*time) < window);

        if self.events.is_empty() {
            return;
        }

        // Calculate events per second
        let event_count = self.events.len() as f64;
        let duration = if self.events.len() > 1 {
            let first = self.events.first().map(|(t, _)| t).unwrap();
            let last = self.events.last().map(|(t, _)| t).unwrap();
            last.duration_since(*first).as_secs_f64().max(0.001)
        } else {
            0.05 // Assume ~20Hz if only one event
        };

        // Calculate velocity as events per second, scaled
        let raw_velocity = event_count / duration;

        // Apply acceleration curve (square root for smoother feel)
        self.velocity =
            (raw_velocity.sqrt() * self.config.base_amount * 2.0).min(self.config.max_velocity);
    }
}

impl Default for MacOSScrollAccel {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollAcceleration for MacOSScrollAccel {
    fn tick(&mut self) -> usize {
        let now = Instant::now();
        let dt = now.duration_since(self.last_tick).as_secs_f64();
        self.last_tick = now;

        if self.events.is_empty() && !self.momentum_active {
            return 0;
        }

        self.calculate_velocity();

        // Calculate lines to scroll
        let lines = (self.velocity * dt * 60.0).round() as usize; // 60 FPS reference

        if self.momentum_active && self.config.momentum_enabled {
            // Apply decay for momentum
            self.velocity *= self.config.decay;

            if self.velocity < self.config.momentum_threshold {
                self.momentum_active = false;
                self.velocity = 0.0;
            }
        }

        lines.max(if self.velocity > 0.0 { 1 } else { 0 })
    }

    fn scroll_event(&mut self, direction: ScrollDirection) {
        let now = Instant::now();
        self.events.push((now, direction));
        self.momentum_active = false; // Reset momentum on new scroll

        // Keep events list bounded
        while self.events.len() > 50 {
            self.events.remove(0);
        }
    }

    fn reset(&mut self) {
        self.events.clear();
        self.velocity = 0.0;
        self.momentum_active = false;
    }

    fn has_momentum(&self) -> bool {
        self.momentum_active && self.velocity > self.config.momentum_threshold
    }
}

/// Scroll state for a scrollable region.
pub struct ScrollState {
    /// Current scroll offset (in lines)
    pub offset: usize,
    /// Total content lines
    pub total_lines: usize,
    /// Visible lines in viewport
    pub visible_lines: usize,
    /// Scroll acceleration handler
    acceleration: Box<dyn ScrollAcceleration + Send>,
    /// Last scroll direction
    last_direction: Option<ScrollDirection>,
}

impl ScrollState {
    pub fn new() -> Self {
        Self {
            offset: 0,
            total_lines: 0,
            visible_lines: 0,
            acceleration: Box::new(MacOSScrollAccel::new()),
            last_direction: None,
        }
    }

    pub fn with_fixed_speed(speed: usize) -> Self {
        Self {
            offset: 0,
            total_lines: 0,
            visible_lines: 0,
            acceleration: Box::new(FixedSpeedScroll::new(speed)),
            last_direction: None,
        }
    }

    /// Handle a scroll event.
    pub fn handle_scroll(&mut self, direction: ScrollDirection) {
        self.last_direction = Some(direction);
        self.acceleration.scroll_event(direction);
    }

    /// Update scroll position based on acceleration. Returns true if position changed.
    pub fn tick(&mut self) -> bool {
        let lines = self.acceleration.tick();
        if lines == 0 {
            return false;
        }

        let old_offset = self.offset;
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);

        match self.last_direction {
            Some(ScrollDirection::Down) => {
                self.offset = (self.offset + lines).min(max_offset);
            }
            Some(ScrollDirection::Up) => {
                self.offset = self.offset.saturating_sub(lines);
            }
            _ => {}
        }

        self.offset != old_offset
    }

    /// Set content dimensions.
    pub fn set_dimensions(&mut self, total_lines: usize, visible_lines: usize) {
        self.total_lines = total_lines;
        self.visible_lines = visible_lines;

        // Clamp offset if content shrunk
        let max_offset = total_lines.saturating_sub(visible_lines);
        if self.offset > max_offset {
            self.offset = max_offset;
        }
    }

    /// Scroll to a specific position.
    pub fn scroll_to(&mut self, offset: usize) {
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);
        self.offset = offset.min(max_offset);
        self.acceleration.reset();
    }

    /// Scroll by a number of lines.
    pub fn scroll_by(&mut self, lines: isize) {
        if lines > 0 {
            let max_offset = self.total_lines.saturating_sub(self.visible_lines);
            self.offset = (self.offset + lines as usize).min(max_offset);
        } else {
            self.offset = self.offset.saturating_sub((-lines) as usize);
        }
    }

    /// Scroll to top.
    pub fn scroll_to_top(&mut self) {
        self.offset = 0;
        self.acceleration.reset();
    }

    /// Scroll to bottom.
    pub fn scroll_to_bottom(&mut self) {
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);
        self.offset = max_offset;
        self.acceleration.reset();
    }

    /// Check if currently at the bottom.
    pub fn is_at_bottom(&self) -> bool {
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);
        self.offset >= max_offset
    }

    /// Get scroll percentage (0.0 - 1.0).
    pub fn scroll_percentage(&self) -> f64 {
        let max_offset = self.total_lines.saturating_sub(self.visible_lines);
        if max_offset == 0 {
            0.0
        } else {
            self.offset as f64 / max_offset as f64
        }
    }
}

impl Default for ScrollState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixed_speed_scroll_basic() {
        let mut scroll = FixedSpeedScroll::new(3);
        assert_eq!(scroll.tick(), 0);

        scroll.scroll_event(ScrollDirection::Down);
        assert_eq!(scroll.tick(), 3);
        assert_eq!(scroll.tick(), 0);
    }

    #[test]
    fn scroll_state_clamps_offset() {
        let mut state = ScrollState::new();
        state.set_dimensions(100, 20);

        state.scroll_to(200);
        assert_eq!(state.offset, 80); // 100 - 20

        state.scroll_to(0);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn scroll_state_scroll_by() {
        let mut state = ScrollState::new();
        state.set_dimensions(100, 20);

        state.scroll_by(10);
        assert_eq!(state.offset, 10);

        state.scroll_by(-5);
        assert_eq!(state.offset, 5);

        state.scroll_by(-100);
        assert_eq!(state.offset, 0);
    }

    #[test]
    fn scroll_state_is_at_bottom() {
        let mut state = ScrollState::new();
        state.set_dimensions(100, 20);

        assert!(!state.is_at_bottom());

        state.scroll_to_bottom();
        assert!(state.is_at_bottom());
    }

    #[test]
    fn macos_scroll_velocity() {
        let mut scroll = MacOSScrollAccel::new();

        // Single event should produce some velocity
        scroll.scroll_event(ScrollDirection::Down);
        scroll.calculate_velocity();
        assert!(scroll.velocity > 0.0);
    }
}

# OpenCode Parallel

A CLI tool for running multiple AI coding agents in parallel, built with Rust.

## Project Structure

- `src/main.rs` - Binary entrypoint
- `src/lib.rs` - Library root
- `src/agent.rs` - Agent definitions
- `src/executor.rs` - Task execution logic
- `src/orchestrator.rs` - Parallel orchestration
- `src/server/` - Server components
- `src/tui/` - Terminal UI (ratatui-based)
- `src/utils.rs` - Shared utilities
- `src/constants.rs` - Project-wide constants

Maintain clear separation of concerns: each module should have a single responsibility. Do not mix UI logic with business logic or execution concerns.

### `src/server/` - Server Components

- `mod.rs` - Module exports and server initialization
- `client.rs` - Client for communicating with opencode instances
- `events.rs` - Event handling and streaming from worker processes
- `logs.rs` - Log collection and management for worker output
- `types.rs` - Shared type definitions for server communication

### `src/tui/` - Terminal UI

- `mod.rs` - Module exports and TUI initialization
- `app.rs` - Core application state and main event loop
- `handlers.rs` - Keyboard and input event handlers
- `commands.rs` - Slash command parsing and execution
- `messages.rs` - Message display and formatting
- `worker.rs` - Worker pane rendering and status display
- `session.rs` - Session management (create, rename, switch, delete)
- `scroll.rs` - Scroll state and navigation logic
- `selection.rs` - Selection state for lists and workers
- `textarea.rs` - Text input area with editing support

### `src/tui/ui/` - UI Rendering

- `mod.rs` - Module exports for UI components
- `render.rs` - Main layout and frame rendering
- `dialogs.rs` - Modal dialogs (permissions, model selector, confirmations)
- `theme.rs` - Color scheme and styling constants

## Development Guidelines

### Separation of Concerns

- Each module must have a single, clear responsibility
- UI logic (`src/tui/`) must not contain business logic or execution concerns
- Server communication (`src/server/`) must not depend on UI or orchestration internals
- Core logic (`agent.rs`, `executor.rs`, `orchestrator.rs`) must remain independent of presentation

### Code Conventions

- Types: `PascalCase`
- Functions/variables: `snake_case`
- Constants: `SCREAMING_SNAKE_CASE`
- All public items must have doc comments

## Testing

Tests must always be written and passing before a solution is considered complete. Run tests with:

```sh
make test
```

For verbose output:

```sh
make test-verbose
```

## Linting

The linter must always be executed and pass before committing any changes. Run the linter with:

```sh
make lint
```

This runs `cargo clippy -- -D warnings`, treating all warnings as errors.

## Formatting

Format code before committing:

```sh
make fmt
```

## Full CI Check

Run all checks locally before pushing:

```sh
make ci
```

This runs formatting, linting, and tests in sequence.

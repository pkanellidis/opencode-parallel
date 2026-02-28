# opencode-parallel

A terminal-based interface for running multiple [opencode](https://opencode.ai) instances in parallel. Orchestrate complex coding tasks by automatically decomposing them into subtasks, each handled by a separate opencode worker running concurrently.

## Features

- **Parallel OpenCode Instances** - Spawn multiple opencode sessions that work simultaneously on different subtasks
- **Interactive TUI** - Navigate between sessions and workers, view real-time streaming output from each instance
- **Smart Orchestration** - Automatically breaks down complex tasks into parallelizable subtasks distributed across opencode workers
- **Permission Handling** - Unified permission dialog for file access across all opencode instances
- **Session Management** - Create, rename, and switch between multiple orchestration sessions
- **Model Selection** - Switch AI providers/models on the fly for all workers

## How It Works

1. You describe a complex task (e.g., "Add unit tests to all service files")
2. The orchestrator analyzes and decomposes it into independent subtasks
3. Each subtask spawns a dedicated opencode instance running in parallel
4. Results are streamed back in real-time and aggregated when complete

## Prerequisites

**You must have opencode installed:**

```bash
# Install opencode
curl -fsSL https://opencode.ai/install | bash

# Or with npm
npm i -g opencode-ai
```

[See opencode installation docs](https://opencode.ai/docs/cli/)

## Installation

```bash
# Install Rust if needed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build from source
git clone https://github.com/yourusername/opencode-parallel.git
cd opencode-parallel
cargo build --release
sudo cp target/release/opencode-parallel /usr/local/bin/
```

## Usage

```bash
# Start the interactive TUI
opencode-parallel

# Or with options
opencode-parallel tui --agents 4 --workdir /path/to/project

# Run batch tasks from config
opencode-parallel run --config tasks.json --parallel 4
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `i` | Enter input mode |
| `j/k` | Navigate/scroll |
| `n/p` | Next/previous session |
| `Tab` | Cycle through workers |
| `l` | Toggle orchestrator logs |
| `d` | Delete selected worker |
| `c` | Clear all workers |
| `Esc` | Back/exit mode |
| `q` | Quit |

## Slash Commands

| Command | Description |
|---------|-------------|
| `/help` | Show all commands |
| `/new [name]` | Create new session |
| `/sessions` | List all sessions |
| `/rename <name>` | Rename current session |
| `/delete` | Delete current session |
| `/models` | List available models |
| `/model` | Open model selector |
| `/reply #N <msg>` | Reply to worker question |
| `/clear` | Clear session messages |

## Batch Processing

Create a `tasks.json` configuration:

```json
{
  "tasks": [
    {
      "provider": "anthropic",
      "model": "claude-sonnet-4-20250514",
      "task": "Refactor authentication module"
    },
    {
      "provider": "openai", 
      "model": "gpt-4",
      "task": "Add unit tests for API handlers"
    }
  ]
}
```

Run with:

```bash
opencode-parallel run --config tasks.json --parallel 4
```

## Development

```bash
# Run in development
cargo run

# Run tests
cargo test

# Build release
cargo build --release
```

## License

This project is licensed under [PolyForm Noncommercial 1.0.0](https://polyformproject.org/licenses/noncommercial/1.0.0/).

You may fork, modify, and share this software for **noncommercial purposes only**. Commercial use requires a separate license.

See [LICENSE](LICENSE) for full terms.

## Acknowledgments

- Built on [opencode](https://opencode.ai)
- TUI powered by [ratatui](https://github.com/ratatui/ratatui)
- Async runtime by [tokio](https://tokio.rs)

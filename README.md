# opencode-parallel

A CLI wrapper for running multiple instances of [opencode](https://opencode.ai) in parallel. Execute multiple coding tasks simultaneously and monitor them all in a native terminal UI.

## What It Does

opencode-parallel wraps the `opencode` CLI and runs multiple instances in parallel:
- Each agent runs as a separate `opencode run` process
- Monitor all agents in real-time with a split-pane TUI
- Execute batch tasks from JSON configuration
- All the power of opencode, parallelized

## Prerequisites

**You must have opencode installed:**

```bash
# Install opencode first
curl -fsSL https://opencode.ai/install | bash

# Or with package managers
brew install opencode
npm i -g opencode-ai
```

[See opencode installation docs](https://opencode.ai/docs/cli/)

## Features

- **🎨 Enhanced TUI**: Beautiful, colorful terminal UI with modern styling
  - Split-pane layout with agent list and detailed output
  - Real-time progress bars for each agent
  - Status icons (▶ Running, ✓ Complete, ✗ Failed, ⏸ Pending)
  - Syntax-highlighted output
- **⚡ Parallel Execution**: Run multiple opencode instances simultaneously
- **💬 Ad-hoc Messaging**: Press 'w' to send messages to individual agents
- **📊 Live Progress Tracking**: See completion progress for each agent
- **🔄 Real-time Updates**: Stream output from each opencode process
- **📦 Batch Processing**: Run predefined task configurations
- **🔑 Uses Your Config**: Leverages your existing opencode authentication

## Installation

### Prerequisites

Install Rust if you haven't already:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### Build from source

```bash
git clone https://github.com/yourusername/opencode-parallel.git
cd opencode-parallel
cargo build --release
sudo cp target/release/opencode-parallel /usr/local/bin/
```

## Quick Start

### 1. Make Sure opencode is Configured

```bash
# Configure opencode with your API keys
opencode auth login

# Test it works
opencode run "Explain Rust ownership"
```

### 2. Start the TUI

```bash
# Run with default 4 parallel agents
opencode-parallel

# Or explicitly start TUI with custom agent count
opencode-parallel tui --agents 8
```

## Keyboard Controls

### Normal Mode
- `q` / `Esc` - Quit application
- `↑` / `k` - Select previous agent
- `↓` / `j` - Select next agent
- `s` - Start selected agent
- `c` - Cancel selected agent
- `w` - Write message to selected agent (enter write mode)
- `u` - Scroll output up
- `d` - Scroll output down

### Write Mode (press 'w' to enter)
- Type your message
- `Enter` - Send message to agent
- `Esc` - Cancel and return to normal mode
- `Backspace` - Delete character

### 3. Batch Processing

Create a task configuration file `tasks.json`:

```json
{
  "tasks": [
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Refactor authentication module"
    },
    {
      "provider": "openai",
      "model": "gpt-4",
      "task": "Add unit tests for API handlers"
    },
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Optimize database queries"
    }
  ]
}
```

Run the batch:

```bash
opencode-parallel run --config tasks.json --parallel 4
```

## Usage

### Commands

```bash
opencode-parallel [COMMAND]

Commands:
  tui         Start the interactive TUI
  run         Run a batch of tasks from a config file
  providers   List configured AI providers
  auth        Configure AI provider credentials
  help        Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

### TUI Keybindings

- `↑/k` - Move selection up
- `↓/j` - Move selection down
- `s` - Start selected agent
- `c` - Cancel selected agent
- `u` - Scroll output up
- `d` - Scroll output down
- `q/Esc` - Quit

## Architecture

```
┌─────────────────┐
│   CLI Entry     │
│   (main.rs)     │
└────────┬────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼──────┐
│  TUI  │ │ Executor│
│ Mode  │ │  Batch  │
└───┬───┘ └──┬──────┘
    │        │
    └────┬───┘
         │
    ┌────▼─────┐
    │  Agent   │
    │ Manager  │
    └──────────┘
```

### Components

- **main.rs**: CLI argument parsing and command routing
- **tui.rs**: Terminal UI with ratatui, split-pane views, and event handling
- **executor.rs**: Parallel task execution and coordination
- **agent.rs**: Agent configuration, status tracking, and provider integration

## Configuration

Configuration is stored in `~/.config/opencode-parallel/`:

- `auth.json` - Provider API keys
- `config.json` - User preferences (coming soon)

## Comparison with opencode

| Feature | opencode | opencode-parallel |
|---------|----------|-------------------|
| Single agent | ✓ | ✓ |
| Multiple agents | - | ✓ |
| TUI | ✓ | ✓ |
| Web UI | ✓ | - |
| Parallel execution | - | ✓ |
| Split-pane view | - | ✓ |
| Language | TypeScript | Rust |

## Roadmap

- [ ] Real AI provider integration (Anthropic, OpenAI, Google)
- [ ] MCP (Model Context Protocol) support
- [ ] Agent-to-agent communication
- [ ] Shared context across agents
- [ ] Result aggregation and merging
- [ ] Session persistence and resume
- [ ] Web dashboard
- [ ] VSCode extension

## Development

```bash
# Run in development mode
cargo run

# Run with specific command
cargo run -- tui --agents 8

# Run tests
cargo test

# Build optimized release
cargo build --release
```

## Documentation

For more detailed information, check out the [docs/](docs/) directory:

- **[QUICKSTART.md](docs/QUICKSTART.md)** - 5-minute getting started guide
- **[ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Technical architecture deep dive
- **[CONTRIBUTING.md](docs/CONTRIBUTING.md)** - Development guidelines
- **[PROJECT_SUMMARY.md](docs/PROJECT_SUMMARY.md)** - Complete project overview
- **[BUILD_COMPLETE.md](docs/BUILD_COMPLETE.md)** - Build success summary
- **[NEXT_STEPS.md](docs/NEXT_STEPS.md)** - What to do next
- **[GIT_SETUP.md](docs/GIT_SETUP.md)** - Git workflow guide
- **[DIAGRAM.txt](docs/DIAGRAM.txt)** - ASCII architecture diagrams

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](docs/CONTRIBUTING.md) before submitting a Pull Request.

## License

MIT License - see [LICENSE](LICENSE) file for details

## Acknowledgments

- Inspired by [opencode](https://github.com/anomalyco/opencode)
- Built with [ratatui](https://github.com/ratatui/ratatui)
- Powered by [tokio](https://tokio.rs)

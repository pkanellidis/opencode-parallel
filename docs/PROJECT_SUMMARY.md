# OpenCode-Parallel Project Summary

## Overview

**opencode-parallel** is a command-line tool built in Rust that enables running multiple AI coding agents in parallel. It features a native terminal UI with split-pane views for monitoring and controlling multiple agents simultaneously.

## Project Information

- **Language**: Rust (Edition 2021)
- **Primary Dependencies**: 
  - ratatui (Terminal UI)
  - tokio (Async runtime)
  - crossterm (Terminal handling)
  - clap (CLI parsing)
- **License**: MIT
- **Version**: 0.1.0

## Key Features

1. **Native Terminal UI (TUI)**
   - Split-pane layout showing agent list and detailed output
   - Real-time status updates
   - Interactive controls (start, cancel, navigate)
   - Vim-style keybindings

2. **Parallel Agent Execution**
   - Run multiple AI agents concurrently
   - Configurable parallelism limit
   - Status tracking (Pending, Running, Completed, Failed, Cancelled)
   - Output capture per agent

3. **Provider Agnostic**
   - Support for multiple AI providers (Anthropic, OpenAI, Google, etc.)
   - Secure credential storage
   - Easy provider configuration

4. **Batch Processing**
   - JSON-based task configuration
   - Sequential and parallel execution
   - Progress reporting

5. **Developer-Friendly CLI**
   - Multiple subcommands
   - Helpful error messages
   - Comprehensive help text

## Architecture

```
opencode-parallel/
├── src/
│   ├── main.rs        # CLI entry point and command routing
│   ├── lib.rs         # Public library interface
│   ├── agent.rs       # Agent state management and auth
│   ├── executor.rs    # Parallel task execution
│   └── tui.rs         # Terminal UI implementation
├── examples/          # Usage examples
├── .github/           # CI/CD workflows
├── Cargo.toml         # Project metadata and dependencies
├── Makefile           # Development shortcuts
├── setup.sh           # Environment setup script
└── demo.sh            # Interactive demo
```

## File Structure

### Core Files

- **main.rs** (67 lines)
  - CLI argument parsing with clap
  - Command dispatching
  - Application initialization

- **agent.rs** (153 lines)
  - `AgentConfig` structure for agent state
  - `AgentStatus` enum for lifecycle tracking
  - Authentication management (auth.json)
  - Provider credential storage

- **executor.rs** (94 lines)
  - Batch task execution
  - Parallel coordination with tokio::JoinSet
  - Progress reporting via channels

- **tui.rs** (273 lines)
  - Ratatui-based terminal UI
  - Split-pane layout (40/60 split)
  - Event handling and input processing
  - Real-time agent status rendering

- **lib.rs** (6 lines)
  - Public API exports
  - Module re-exports for library usage

### Configuration

- **Cargo.toml** (40 lines)
  - Binary and library configuration
  - Dependencies specification
  - Release profile optimization

### Documentation

- **README.md** - Primary documentation (300+ lines)
- **ARCHITECTURE.md** - Technical architecture details (400+ lines)
- **CONTRIBUTING.md** - Contribution guidelines (300+ lines)
- **QUICKSTART.md** - Getting started guide (200+ lines)
- **LICENSE** - MIT license

### Development Tools

- **Makefile** - Build automation (15 targets)
- **setup.sh** - Development environment setup
- **demo.sh** - Interactive demonstration
- **.github/workflows/** - CI/CD automation
  - ci.yml - Tests, linting, security audit
  - release.yml - Multi-platform binary builds

### Examples

- **tasks.example.json** - Sample batch configuration
- **examples/basic_usage.rs** - Library usage example

## Commands

### CLI Commands

```bash
opencode-parallel                    # Start TUI (default)
opencode-parallel tui [OPTIONS]      # Start TUI explicitly
opencode-parallel run --config FILE  # Batch execution
opencode-parallel providers          # List configured providers
opencode-parallel auth PROVIDER      # Configure authentication
opencode-parallel --help             # Show help
opencode-parallel --version          # Show version
```

### TUI Keybindings

- `↑` or `k` - Navigate up
- `↓` or `j` - Navigate down
- `s` - Start agent
- `c` - Cancel agent
- `u` - Scroll output up
- `d` - Scroll output down
- `q` or `Esc` - Quit

### Make Targets

```bash
make help        # Show available commands
make build       # Build project
make test        # Run tests
make lint        # Run clippy
make fmt         # Format code
make release     # Build optimized binary
make install     # Install to system
make clean       # Clean artifacts
make ci          # Run all checks
```

## Technical Highlights

### Concurrency Model

- **Async Runtime**: Tokio for efficient async/await
- **Task Management**: JoinSet for parallel agent execution
- **Message Passing**: mpsc channels for UI updates
- **Non-blocking I/O**: All operations are async

### UI Design

- **Layout**: Horizontal split with ratatui constraints
- **Rendering**: Event-driven updates at 100ms intervals
- **State Management**: Centralized App struct
- **Status Indicators**: Unicode symbols for visual clarity

### Code Quality

- **Error Handling**: anyhow::Result throughout
- **Type Safety**: Strong typing with enums and structs
- **Serialization**: serde for JSON handling
- **Documentation**: Comprehensive inline docs

### Build Configuration

- **Release Optimization**:
  - Strip symbols
  - LTO (Link Time Optimization)
  - opt-level = "z" (size optimization)
  - Single codegen unit

## Development Status

### Implemented ✅

- [x] CLI framework with clap
- [x] Terminal UI with ratatui
- [x] Split-pane layout
- [x] Agent state management
- [x] Parallel execution framework
- [x] Batch processing
- [x] Authentication storage
- [x] Interactive controls
- [x] Status indicators
- [x] Example configurations
- [x] CI/CD workflows
- [x] Documentation

### Simulated (Placeholders) 🔄

- [~] AI provider integration (currently simulated)
- [~] Actual API calls (using mock delays)
- [~] Streaming output (placeholder logic)

### Planned 📋

- [ ] Real AI provider implementations
- [ ] MCP (Model Context Protocol) support
- [ ] Agent-to-agent communication
- [ ] Shared context across agents
- [ ] Result aggregation
- [ ] Session persistence
- [ ] Web dashboard
- [ ] VSCode extension

## Getting Started

### Quick Start

```bash
# Clone and setup
git clone <repo>
cd opencode-parallel
./setup.sh

# Run demo
./demo.sh

# Or build and run directly
cargo build --release
./target/release/opencode-parallel
```

### For Developers

```bash
# Run tests
make test

# Check code quality
make ci

# Build and install
make install
```

## Dependencies

### Runtime Dependencies

- **ratatui** (0.28) - Terminal UI framework
- **crossterm** (0.28) - Cross-platform terminal manipulation
- **tokio** (1.41) - Async runtime with full features
- **serde** (1.0) - Serialization framework
- **serde_json** (1.0) - JSON support
- **clap** (4.5) - CLI argument parsing
- **anyhow** (1.0) - Error handling
- **reqwest** (0.12) - HTTP client (for future use)
- **futures** (0.3) - Future utilities
- **chrono** (0.4) - Date/time handling
- **uuid** (1.11) - Unique identifiers

### Total Line Count

- Rust code: ~590 lines
- Documentation: ~1200+ lines
- Configuration: ~100 lines
- Scripts: ~200 lines

## Performance Characteristics

- **Startup Time**: < 100ms
- **Memory Usage**: ~10-20MB base
- **UI Refresh Rate**: 10 FPS (100ms)
- **Max Parallel Agents**: Configurable (default: 4)
- **Build Time**: ~2-3 minutes (initial), <30s (incremental)
- **Binary Size**: ~2-3MB (release, stripped)

## Platform Support

- **Linux**: x86_64, aarch64
- **macOS**: x86_64 (Intel), aarch64 (Apple Silicon)
- **Windows**: x86_64

## Similar Projects

- **opencode**: Inspiration - single agent TUI (TypeScript)
- **aider**: AI pair programming (Python)
- **cursor**: AI code editor (Electron)
- **GitHub Copilot**: AI completion (VS Code extension)

## Unique Selling Points

1. **Truly Parallel**: Run multiple agents simultaneously
2. **Native Performance**: Rust implementation, fast startup
3. **Visual Management**: See all agents at once
4. **Provider Agnostic**: Not tied to single AI provider
5. **Lightweight**: Small binary, minimal dependencies
6. **Open Source**: MIT licensed, community-driven

## Future Vision

Transform opencode-parallel into a comprehensive platform for:
- Distributed AI agent orchestration
- Multi-model task decomposition
- Collaborative agent workflows
- Real-time monitoring and debugging
- Integration with existing dev tools

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) file.

---

**Status**: Development/Demo Version  
**Last Updated**: February 2026  
**Maintainer**: OpenCode Parallel Contributors

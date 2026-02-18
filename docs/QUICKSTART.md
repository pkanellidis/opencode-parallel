# Quick Start Guide

Get up and running with opencode-parallel in 5 minutes.

## Installation

### Option 1: From Source (Recommended)

```bash
# Clone the repository
git clone https://github.com/yourusername/opencode-parallel.git
cd opencode-parallel

# Run setup script (installs Rust if needed)
./setup.sh

# Build and install
make install
```

### Option 2: Using Cargo

```bash
cargo install opencode-parallel
```

## First Run

### 1. Start the TUI

```bash
opencode-parallel
```

This starts the interactive Terminal UI with 4 parallel agents.

### 2. Navigate the Interface

```
┌─────────────┬───────────────────────────────┐
│             │                               │
│   Agents    │     Selected Agent            │
│             │     Output & Details          │
│             │                               │
│  ○ Agent 1  │  Waiting to start...          │
│  ◉ Agent 2  │                               │
│  ✓ Agent 3  │                               │
│  ✗ Agent 4  │                               │
│             │                               │
└─────────────┴───────────────────────────────┘
 q:Quit | ↑/k:Up | ↓/j:Down | s:Start | c:Cancel
```

**Controls:**
- `↑` or `k` - Move selection up
- `↓` or `j` - Move selection down
- `s` - Start selected agent
- `c` - Cancel running agent
- `u` - Scroll output up
- `d` - Scroll output down
- `q` or `Esc` - Quit

### 3. Configure Providers

Before running real AI agents, configure your API keys:

```bash
# Anthropic Claude
opencode-parallel auth anthropic --key sk-ant-api03-...

# OpenAI GPT
opencode-parallel auth openai --key sk-...

# Google Gemini
opencode-parallel auth google --key ...

# List configured providers
opencode-parallel providers
```

## Running Batch Tasks

### 1. Create a Task Configuration

Create `my-tasks.json`:

```json
{
  "tasks": [
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Add error handling to user service"
    },
    {
      "provider": "openai",
      "model": "gpt-4",
      "task": "Write integration tests for API"
    },
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Refactor database queries"
    }
  ]
}
```

### 2. Run the Batch

```bash
opencode-parallel run --config my-tasks.json --parallel 3
```

This will execute all tasks in parallel with a maximum of 3 concurrent agents.

## Common Use Cases

### Refactoring Multiple Files

```json
{
  "tasks": [
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Refactor src/auth.rs to use async/await"
    },
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Refactor src/database.rs to use connection pooling"
    },
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Refactor src/api.rs to add error handling"
    }
  ]
}
```

### Writing Tests

```json
{
  "tasks": [
    {
      "provider": "openai",
      "model": "gpt-4",
      "task": "Write unit tests for user authentication"
    },
    {
      "provider": "openai",
      "model": "gpt-4",
      "task": "Write integration tests for API endpoints"
    },
    {
      "provider": "openai",
      "model": "gpt-4",
      "task": "Write end-to-end tests for user flows"
    }
  ]
}
```

### Code Review

```json
{
  "tasks": [
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Review authentication module for security issues"
    },
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Review database queries for SQL injection"
    },
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Review API handlers for error handling"
    }
  ]
}
```

## Tips & Tricks

### 1. Optimize Parallel Count

Start with fewer parallel agents and increase:

```bash
# Start conservative
opencode-parallel tui --agents 2

# Scale up as needed
opencode-parallel tui --agents 8
```

### 2. Use Different Models

Mix and match models for different tasks:

```json
{
  "tasks": [
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Complex refactoring"
    },
    {
      "provider": "openai",
      "model": "gpt-3.5-turbo",
      "task": "Simple documentation"
    }
  ]
}
```

### 3. Check Status Indicators

- `○` - Pending (not started)
- `◉` - Running (in progress)
- `✓` - Completed (success)
- `✗` - Failed (error)
- `⊘` - Cancelled (by user)

### 4. Scroll Through Output

When viewing agent output:
- Press `d` to scroll down
- Press `u` to scroll up

## Troubleshooting

### "Rust not found"

Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### "Provider not configured"

Configure authentication:
```bash
opencode-parallel auth <provider-name>
```

### "Build failed"

Clean and rebuild:
```bash
make clean
make build
```

### TUI Not Displaying Correctly

Ensure your terminal supports:
- 256 colors
- Unicode characters
- Minimum 80x24 size

## Next Steps

- Read the [Architecture Guide](ARCHITECTURE.md)
- Check out [Contributing Guidelines](CONTRIBUTING.md)
- Browse [examples/](examples/) directory
- Join our community (coming soon)

## Getting Help

- GitHub Issues: Report bugs or request features
- Documentation: Read the full [README.md](README.md)
- Examples: Check `examples/` directory

---

Happy parallel coding! 🚀

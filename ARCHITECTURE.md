# Architecture

## Overview

opencode-parallel is designed as a modular Rust CLI application that orchestrates multiple AI coding agents running in parallel. The architecture emphasizes concurrency, clean separation of concerns, and extensibility.

## Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                         CLI Layer                            │
│                        (main.rs)                            │
│  ┌──────────┬──────────┬──────────┬──────────────────┐    │
│  │   tui    │   run    │   auth   │   providers      │    │
│  └──────────┴──────────┴──────────┴──────────────────┘    │
└───────┬─────────┬───────────┬──────────┬───────────────────┘
        │         │           │          │
┌───────▼─────┐ ┌─▼─────────┐ ┌────▼────┐ ┌─▼──────────┐
│ TUI Module  │ │ Executor  │ │ Agent   │ │ Provider   │
│ (tui.rs)    │ │ Module    │ │ Manager │ │ Manager    │
│             │ │(executor  │ │(agent   │ │            │
│             │ │   .rs)    │ │  .rs)   │ │            │
└─────┬───────┘ └─────┬─────┘ └────┬────┘ └────────────┘
      │               │             │
      │         ┌─────▼─────────────▼───────────────┐
      │         │       Agent Execution Pool         │
      │         │  (tokio::task::JoinSet)           │
      │         └───────────────────────────────────┘
      │
┌─────▼──────────────────────────────────────────────────┐
│            Ratatui Rendering Layer                      │
│  ┌──────────────┐  ┌────────────────────────────┐     │
│  │ Agent List   │  │  Agent Detail Pane         │     │
│  │ Pane         │  │  (Output, Logs, Status)    │     │
│  └──────────────┘  └────────────────────────────┘     │
└────────────────────────────────────────────────────────┘
```

## Module Breakdown

### 1. Main Module (`main.rs`)

**Responsibilities:**
- CLI argument parsing using `clap`
- Command routing to appropriate modules
- Application initialization

**Key Functions:**
- `main()` - Entry point, tokio runtime initialization
- Command handling for: `tui`, `run`, `auth`, `providers`

### 2. Agent Module (`agent.rs`)

**Responsibilities:**
- Agent lifecycle management
- Status tracking (Pending, Running, Completed, Failed, Cancelled)
- Provider authentication configuration
- Agent state serialization

**Key Structures:**
```rust
AgentConfig {
    id: String,
    provider: String,
    model: String,
    task: String,
    status: AgentStatus,
    output: Vec<String>,
    started_at: Option<DateTime>,
    completed_at: Option<DateTime>,
}
```

**Key Functions:**
- `configure_auth()` - Store provider credentials
- `get_provider_key()` - Retrieve API keys
- `list_providers()` - Show configured providers

### 3. Executor Module (`executor.rs`)

**Responsibilities:**
- Batch task execution
- Parallel agent coordination
- Task queue management
- Progress reporting

**Key Functions:**
- `run_batch()` - Execute multiple tasks with controlled parallelism
- `run_agent()` - Execute individual agent task
- Task result aggregation

**Concurrency Model:**
```rust
// Uses tokio::task::JoinSet for managing parallel tasks
let mut set = JoinSet::new();
for task in tasks {
    set.spawn(async move {
        run_agent(task).await
    });
}
```

### 4. TUI Module (`tui.rs`)

**Responsibilities:**
- Terminal UI rendering
- User input handling
- Real-time agent status updates
- Split-pane layout management

**Layout Structure:**
```
┌─────────────┬───────────────────────────────┐
│             │                               │
│   Agent     │     Selected Agent            │
│   List      │     Output & Details          │
│   (40%)     │     (60%)                     │
│             │                               │
│   ○ Agent 1 │  Starting task...             │
│   ◉ Agent 2 │  Processing step 1...         │
│   ✓ Agent 3 │  Processing step 2...         │
│   ✗ Agent 4 │  Task completed!              │
│             │                               │
└─────────────┴───────────────────────────────┘
 q:Quit | ↑/k:Up | ↓/j:Down | s:Start | c:Cancel
```

**Event Loop:**
1. Poll for keyboard events (100ms timeout)
2. Process user input
3. Update agent state from background tasks
4. Render UI frame
5. Repeat

### 5. Provider Integration (Future)

```rust
// Future module structure
providers/
  ├── anthropic.rs
  ├── openai.rs
  ├── google.rs
  └── local.rs

trait Provider {
    async fn execute(&self, task: &str) -> Result<String>;
    fn supports_streaming(&self) -> bool;
}
```

## Data Flow

### TUI Mode

```
User Input
    │
    ▼
Event Handler
    │
    ├─► Start Agent ──► spawn(run_agent) ──► Agent Pool
    │
    ├─► Cancel Agent ──► send cancel signal
    │
    └─► Navigate ──► update selected index
                        │
                        ▼
                    Update State
                        │
                        ▼
                    Render Frame
                        │
                        ▼
                   Terminal Output
```

### Batch Mode

```
Task Config (JSON)
    │
    ▼
Parse Tasks
    │
    ▼
Create Agent Pool (size = max_parallel)
    │
    ├─► Agent 1 ──► API Call ──► Stream Results
    ├─► Agent 2 ──► API Call ──► Stream Results
    ├─► Agent 3 ──► API Call ──► Stream Results
    └─► Agent 4 ──► API Call ──► Stream Results
           │
           ▼
    Aggregate Results
           │
           ▼
    Display Summary
```

## State Management

### Agent State Machine

```
    [Pending]
        │
        │ start()
        ▼
    [Running] ────┐
        │         │ cancel()
        │         │
        │         ▼
        │    [Cancelled]
        │
        │ complete()
        ▼
    [Completed]
        
        │ fail()
        ▼
    [Failed]
```

### State Synchronization

The TUI uses message passing for state updates:

```rust
// Producer (agent tasks)
tx.send(AgentUpdate { id, status, output }).await;

// Consumer (TUI event loop)
while let Ok(update) = rx.try_recv() {
    // Update agent state
}
```

## Configuration

### File Locations

```
~/.config/opencode-parallel/
  ├── auth.json           # API keys
  └── config.json         # User preferences (future)
```

### Authentication Storage

```json
{
  "providers": {
    "anthropic": "sk-ant-...",
    "openai": "sk-...",
    "google": "..."
  }
}
```

## Concurrency Design

### Tokio Runtime

- Uses multi-threaded tokio runtime
- Each agent runs as separate async task
- Non-blocking I/O for all operations

### Parallelism Control

```rust
// Semaphore-like behavior using JoinSet
let mut active = 0;
while active < max_parallel {
    if let Some(task) = tasks.next() {
        set.spawn(run_agent(task));
        active += 1;
    }
}
```

## Error Handling

All modules use `anyhow::Result<T>` for error propagation:

```rust
pub async fn run_agent(agent: AgentConfig) -> Result<AgentConfig> {
    // Operations that may fail
    agent.validate()?;
    let api_key = get_provider_key(&agent.provider)?;
    // ...
}
```

## Testing Strategy

```rust
// Unit tests per module
#[cfg(test)]
mod tests {
    #[test]
    fn test_agent_state_transitions() { }
    
    #[tokio::test]
    async fn test_parallel_execution() { }
}
```

## Performance Considerations

1. **Memory**: Bounded channels prevent unbounded memory growth
2. **CPU**: Tasks yield to runtime, preventing blocking
3. **I/O**: All I/O is async (file, network, terminal)
4. **Rendering**: TUI updates at 10 FPS (100ms poll interval)

## Future Enhancements

1. **Real AI Integration**: Replace simulated work with actual API calls
2. **Session Persistence**: Save/restore agent state
3. **Inter-agent Communication**: Shared context and coordination
4. **Web Dashboard**: Alternative UI via HTTP server
5. **Plugin System**: Custom agent types and providers
6. **Distributed Execution**: Run agents across multiple machines

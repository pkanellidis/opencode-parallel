# How opencode-parallel Works

## Architecture Overview

opencode-parallel is a **CLI wrapper** that orchestrates multiple instances of the `opencode` CLI running in parallel.

```
┌─────────────────────────────────────────────────┐
│         opencode-parallel                        │
│         (Orchestrator)                           │
└──────────┬───────────┬───────────┬──────────────┘
           │           │           │
           ▼           ▼           ▼
    ┌──────────┐ ┌──────────┐ ┌──────────┐
    │opencode  │ │opencode  │ │opencode  │
    │   run    │ │   run    │ │   run    │
    │ Task 1   │ │ Task 2   │ │ Task 3   │
    └──────────┘ └──────────┘ └──────────┘
           │           │           │
           ▼           ▼           ▼
       AI API      AI API      AI API
```

## How It Works

### 1. Task Definition

You define tasks in JSON:

```json
{
  "tasks": [
    {
      "provider": "anthropic",
      "model": "claude-3-5-sonnet-20241022",
      "task": "Explain Rust ownership"
    }
  ]
}
```

### 2. Process Spawning

For each task, opencode-parallel spawns a subprocess:

```rust
Command::new("opencode")
    .arg("run")
    .arg(&task)
    .arg("--model")
    .arg("anthropic/claude-3-5-sonnet-20241022")
    .spawn()
```

### 3. Output Streaming

Each process's stdout is streamed in real-time:

```rust
let stdout = child.stdout.take()?;
let mut reader = BufReader::new(stdout).lines();

while let Ok(Some(line)) = reader.next_line().await {
    // Display in TUI
    agent.add_output(line);
}
```

### 4. Parallel Coordination

Using tokio's JoinSet to manage multiple concurrent tasks:

```rust
let mut set = JoinSet::new();

for task in tasks {
    set.spawn(async move {
        run_opencode_process(task).await
    });
}
```

## Key Components

### Executor (src/executor.rs)

- Spawns opencode processes
- Manages parallelism limits
- Streams output from subprocesses
- Tracks completion status

### TUI (src/tui.rs)

- Displays all running agents
- Split-pane view (agent list + details)
- Real-time output updates
- Interactive controls

### Agent Manager (src/agent.rs)

- Tracks agent state (Pending, Running, Completed, Failed)
- Stores output lines
- Calculates duration
- No AI integration - just metadata

## Data Flow

```
User Input
    │
    ▼
Parse JSON Config
    │
    ▼
Create Agent Configs
    │
    ▼
For each agent:
    ├─► Spawn opencode process
    ├─► Stream stdout/stderr
    ├─► Update agent state
    └─► Display in TUI
    │
    ▼
Wait for all to complete
    │
    ▼
Show summary
```

## Authentication

opencode-parallel **does NOT** handle authentication itself. It relies on:

1. **opencode's configuration** - Uses your existing opencode auth setup
2. **Environment variables** - Any API keys in your environment
3. **Config files** - Your ~/.local/share/opencode/auth.json

This means:
- ✅ No duplicate auth management
- ✅ Single source of truth (opencode)
- ✅ All opencode features work (providers, models, etc.)

## Comparison with Standalone Implementation

### Original Plan (Standalone)
```
opencode-parallel → Direct API calls → AI Providers
```
- Would need to implement all API integrations
- Would duplicate opencode's functionality
- Would need separate authentication

### Current Implementation (Wrapper)
```
opencode-parallel → opencode CLI → AI Providers
```
- ✅ Leverages existing opencode
- ✅ All opencode features available
- ✅ Simpler implementation
- ✅ Automatic updates when opencode updates

## Benefits of This Approach

1. **Simplicity** - No need to reimplement AI integrations
2. **Compatibility** - Works with all opencode features
3. **Updates** - Automatically gets opencode improvements
4. **Authentication** - Uses existing setup
5. **Focus** - Can focus on parallelization, not AI APIs

## Example: Running 3 Tasks in Parallel

```bash
# Define tasks
cat > tasks.json << 'EOF'
{
  "tasks": [
    {"task": "Explain async/await", ...},
    {"task": "Write a web server", ...},
    {"task": "Implement sorting", ...}
  ]
}
EOF

# Run in parallel (max 3 at once)
opencode-parallel run --config tasks.json --parallel 3
```

Behind the scenes:
1. Parse JSON → 3 task definitions
2. Spawn 3 opencode processes simultaneously
3. Stream all 3 outputs in real-time
4. Display in TUI with split panes
5. Wait for all to complete
6. Show summary

## Future Enhancements

Possible additions while keeping the wrapper approach:

1. **Result Aggregation** - Combine outputs from multiple agents
2. **Dependency Management** - Run tasks in order based on dependencies
3. **Cost Tracking** - Parse opencode output for token usage
4. **Session Persistence** - Save and restore parallel sessions
5. **Web Dashboard** - Web UI for monitoring
6. **Agent Communication** - Share context between agents

All while still using opencode for the actual AI work!

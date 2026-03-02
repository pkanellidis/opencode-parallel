# OpenCode Parallel Server (TypeScript)

A TypeScript server for running multiple opencode AI agents in parallel.

## Installation

```bash
cd ts-server
npm install
```

## Usage

### Start the Server

```bash
npm run dev
```

Environment variables:
- `PORT` - Server port (default: 3000)
- `BASE_PORT` - Starting port for workers (default: 4000)
- `MAX_WORKERS` - Maximum number of workers (default: 10)
- `WORKDIR` - Default working directory for workers

### API Endpoints

#### Health Check
```
GET /api/health
```

#### Workers

```
GET  /api/workers              # List all workers
POST /api/workers              # Create and start a worker
GET  /api/workers/:id          # Get worker status
DELETE /api/workers/:id        # Stop and remove worker
POST /api/workers/:id/start    # Start a worker
POST /api/workers/:id/stop     # Stop a worker
POST /api/workers/:id/session  # Create a new session
POST /api/workers/:id/message  # Send message (blocking)
POST /api/workers/:id/message/stream  # Send message (streaming)
```

#### Tasks

```
POST /api/tasks           # Execute single task
POST /api/tasks/parallel  # Execute multiple tasks in parallel
POST /api/tasks/stream    # Execute task with streaming events
```

### SDK Usage

```typescript
import { OpenCodeClient, Orchestrator } from "opencode-parallel-server";

// Direct client usage
const client = new OpenCodeClient(4000);
await client.waitForHealth();
const session = await client.createSession();
const response = await client.sendMessage(session.id, { content: "Hello" });

// Orchestrator usage
const orchestrator = new Orchestrator({
  basePort: 4000,
  maxWorkers: 5,
});

const worker = await orchestrator.createWorker("worker-1");
await worker.start();
await worker.createSession();

const result = await orchestrator.executeTask({
  workerId: "worker-1",
  prompt: "Write a function",
});
```

## Development

```bash
npm run build      # Build TypeScript
npm run lint       # Run ESLint
npm run typecheck  # Run type checking
npm test           # Run tests
```

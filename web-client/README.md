# OpenCode Parallel Web Client

A web interface for the OpenCode Parallel functionality built with React and TypeScript.

## Prerequisites

- Node.js 18+
- OpenCode server running on port 14096

## Development

```bash
# Install dependencies
npm install

# Start development server
npm run dev
```

The dev server runs on http://localhost:3000 and proxies API requests to the OpenCode server.

## Production Build

```bash
npm run build
```

Built files are output to the `dist/` directory.

## Features

- Real-time streaming via Server-Sent Events (SSE)
- Session management with multiple parallel workers
- Model selection from available providers
- Interactive question/permission handling
- Markdown rendering for AI responses
- Tool execution tracking

## Architecture

- `src/api/client.ts` - HTTP client matching the Rust server API
- `src/hooks/useOpenCode.ts` - Main React hook for state management
- `src/components/` - UI components
- `src/types/` - TypeScript type definitions matching server types

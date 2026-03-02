import express from "express";
import { Orchestrator } from "./server/orchestrator.js";
import { createRoutes } from "./server/routes.js";

export * from "./types/index.js";
export * from "./sdk/index.js";
export * from "./server/index.js";

const PORT = parseInt(process.env.PORT ?? "3000", 10);
const BASE_PORT = parseInt(process.env.BASE_PORT ?? "4000", 10);
const MAX_WORKERS = parseInt(process.env.MAX_WORKERS ?? "10", 10);
const WORKDIR = process.env.WORKDIR;

async function main(): Promise<void> {
  const orchestrator = new Orchestrator({
    basePort: BASE_PORT,
    maxWorkers: MAX_WORKERS,
    workdir: WORKDIR,
  });

  const app = express();
  app.use(express.json());
  app.use("/api", createRoutes(orchestrator));

  const shutdown = async (): Promise<void> => {
    console.log("\nShutting down...");
    await orchestrator.shutdown();
    process.exit(0);
  };

  process.on("SIGINT", shutdown);
  process.on("SIGTERM", shutdown);

  app.listen(PORT, () => {
    console.log(`OpenCode Parallel Server running on http://localhost:${PORT}`);
    console.log(`  Base port for workers: ${BASE_PORT}`);
    console.log(`  Max workers: ${MAX_WORKERS}`);
    if (WORKDIR) {
      console.log(`  Working directory: ${WORKDIR}`);
    }
  });
}

main().catch((err) => {
  console.error("Failed to start server:", err);
  process.exit(1);
});

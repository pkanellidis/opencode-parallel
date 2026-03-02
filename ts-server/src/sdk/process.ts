import { spawn, ChildProcess } from "child_process";
import { OpenCodeClient } from "./client.js";

export interface ServerProcessOptions {
  port: number;
  workdir?: string;
  healthTimeoutMs?: number;
}

export class ServerProcess {
  private child: ChildProcess | null = null;
  private port: number;
  private workdir?: string;
  private healthTimeoutMs: number;

  constructor(options: ServerProcessOptions) {
    this.port = options.port;
    this.workdir = options.workdir;
    this.healthTimeoutMs = options.healthTimeoutMs ?? 30000;
  }

  async start(): Promise<OpenCodeClient> {
    const args = ["serve", "--port", this.port.toString()];

    this.child = spawn("opencode", args, {
      cwd: this.workdir,
      stdio: ["ignore", "pipe", "pipe"],
      detached: false,
    });

    const client = new OpenCodeClient(this.port);

    this.child.on("error", (err) => {
      console.error(`Server process error: ${err.message}`);
    });

    this.child.stderr?.on("data", (data: Buffer) => {
      const text = data.toString();
      if (text.trim()) {
        console.error(`[opencode:${this.port}] ${text.trim()}`);
      }
    });

    const healthy = await client.waitForHealth(this.healthTimeoutMs);
    if (!healthy) {
      this.stop();
      throw new Error(
        `Server failed to become healthy within ${this.healthTimeoutMs}ms`
      );
    }

    return client;
  }

  stop(): void {
    if (this.child) {
      this.child.kill("SIGTERM");
      this.child = null;
    }
  }

  get isRunning(): boolean {
    return this.child !== null && this.child.exitCode === null;
  }

  get processPort(): number {
    return this.port;
  }
}

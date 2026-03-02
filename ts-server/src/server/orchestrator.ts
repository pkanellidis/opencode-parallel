import { Worker } from "./worker.js";
import type {
  WorkerConfig,
  WorkerStatus,
  TaskRequest,
  TaskResult,
  StreamEvent,
} from "../types/index.js";

export interface OrchestratorConfig {
  basePort: number;
  maxWorkers: number;
  workdir?: string;
}

export class Orchestrator {
  private workers: Map<string, Worker> = new Map();
  private config: OrchestratorConfig;
  private nextPort: number;

  constructor(config: OrchestratorConfig) {
    this.config = config;
    this.nextPort = config.basePort;
  }

  async createWorker(id: string, workdir?: string): Promise<Worker> {
    if (this.workers.size >= this.config.maxWorkers) {
      throw new Error(`Maximum workers (${this.config.maxWorkers}) reached`);
    }

    if (this.workers.has(id)) {
      throw new Error(`Worker with id '${id}' already exists`);
    }

    const workerConfig: WorkerConfig = {
      id,
      port: this.nextPort++,
      workdir: workdir ?? this.config.workdir,
    };

    const worker = new Worker(workerConfig);
    this.workers.set(id, worker);

    return worker;
  }

  async startWorker(id: string): Promise<void> {
    const worker = this.workers.get(id);
    if (!worker) {
      throw new Error(`Worker '${id}' not found`);
    }
    await worker.start();
  }

  async stopWorker(id: string): Promise<void> {
    const worker = this.workers.get(id);
    if (!worker) {
      throw new Error(`Worker '${id}' not found`);
    }
    await worker.stop();
  }

  async removeWorker(id: string): Promise<void> {
    const worker = this.workers.get(id);
    if (worker) {
      await worker.stop();
      this.workers.delete(id);
    }
  }

  getWorker(id: string): Worker | undefined {
    return this.workers.get(id);
  }

  getAllWorkers(): Worker[] {
    return Array.from(this.workers.values());
  }

  getWorkerStatuses(): WorkerStatus[] {
    return this.getAllWorkers().map((w) => w.getStatus());
  }

  async executeTask(request: TaskRequest): Promise<TaskResult> {
    const worker = this.workers.get(request.workerId);
    if (!worker) {
      return {
        workerId: request.workerId,
        success: false,
        error: `Worker '${request.workerId}' not found`,
      };
    }

    try {
      if (!worker.isRunning) {
        await worker.start();
      }

      const status = worker.getStatus();
      if (!status.sessionId) {
        await worker.createSession();
      }

      const response = await worker.sendMessage({
        content: request.prompt,
        model: request.model,
      });

      return {
        workerId: request.workerId,
        success: true,
        response,
      };
    } catch (err) {
      return {
        workerId: request.workerId,
        success: false,
        error: err instanceof Error ? err.message : String(err),
      };
    }
  }

  async executeTasksParallel(requests: TaskRequest[]): Promise<TaskResult[]> {
    return Promise.all(requests.map((req) => this.executeTask(req)));
  }

  async executeTaskAsync(
    request: TaskRequest,
    onEvent: (workerId: string, event: StreamEvent) => void
  ): Promise<TaskResult> {
    const worker = this.workers.get(request.workerId);
    if (!worker) {
      return {
        workerId: request.workerId,
        success: false,
        error: `Worker '${request.workerId}' not found`,
      };
    }

    try {
      if (!worker.isRunning) {
        await worker.start();
      }

      const status = worker.getStatus();
      if (!status.sessionId) {
        await worker.createSession();
      }

      await worker.sendMessageAsync(
        { content: request.prompt, model: request.model },
        (event) => onEvent(request.workerId, event)
      );

      return {
        workerId: request.workerId,
        success: true,
      };
    } catch (err) {
      return {
        workerId: request.workerId,
        success: false,
        error: err instanceof Error ? err.message : String(err),
      };
    }
  }

  async stopAll(): Promise<void> {
    const stopPromises = this.getAllWorkers().map((w) => w.stop());
    await Promise.all(stopPromises);
  }

  async shutdown(): Promise<void> {
    await this.stopAll();
    this.workers.clear();
  }
}

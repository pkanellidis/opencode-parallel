import { OpenCodeClient, ServerProcess } from "../sdk/index.js";
import type {
  WorkerConfig,
  WorkerStatus,
  SendMessageRequest,
  MessageResponse,
  StreamEvent,
} from "../types/index.js";

export class Worker {
  private config: WorkerConfig;
  private process: ServerProcess | null = null;
  private client: OpenCodeClient | null = null;
  private sessionId: string | null = null;
  private status: WorkerStatus["status"] = "stopped";
  private lastError?: string;
  private eventUnsubscribe?: () => void;

  constructor(config: WorkerConfig) {
    this.config = config;
  }

  async start(): Promise<void> {
    if (this.process?.isRunning) {
      return;
    }

    try {
      this.process = new ServerProcess({
        port: this.config.port,
        workdir: this.config.workdir,
      });

      this.client = await this.process.start();
      this.status = "idle";
      this.lastError = undefined;
    } catch (err) {
      this.status = "error";
      this.lastError = err instanceof Error ? err.message : String(err);
      throw err;
    }
  }

  async stop(): Promise<void> {
    if (this.eventUnsubscribe) {
      this.eventUnsubscribe();
      this.eventUnsubscribe = undefined;
    }

    this.process?.stop();
    this.process = null;
    this.client = null;
    this.sessionId = null;
    this.status = "stopped";
  }

  async createSession(): Promise<string> {
    if (!this.client) {
      throw new Error("Worker not started");
    }

    const session = await this.client.createSession();
    this.sessionId = session.id;
    return session.id;
  }

  async sendMessage(request: SendMessageRequest): Promise<MessageResponse> {
    if (!this.client || !this.sessionId) {
      throw new Error("Worker not started or no session");
    }

    this.status = "running";

    try {
      const response = await this.client.sendMessage(this.sessionId, {
        ...request,
        model: request.model ?? this.config.model,
      });
      this.status = "idle";
      return response;
    } catch (err) {
      this.status = "error";
      this.lastError = err instanceof Error ? err.message : String(err);
      throw err;
    }
  }

  async sendMessageAsync(
    request: SendMessageRequest,
    onEvent: (event: StreamEvent) => void
  ): Promise<void> {
    if (!this.client || !this.sessionId) {
      throw new Error("Worker not started or no session");
    }

    this.status = "running";

    try {
      await this.client.sendMessageAsync(
        this.sessionId,
        {
          ...request,
          model: request.model ?? this.config.model,
        },
        onEvent
      );
      this.status = "idle";
    } catch (err) {
      this.status = "error";
      this.lastError = err instanceof Error ? err.message : String(err);
      throw err;
    }
  }

  subscribeToEvents(onEvent: (event: StreamEvent) => void): void {
    if (!this.client) {
      throw new Error("Worker not started");
    }

    if (this.eventUnsubscribe) {
      this.eventUnsubscribe();
    }

    this.eventUnsubscribe = this.client.subscribeToEvents(onEvent);
  }

  async replyToQuestion(requestId: string, answers: string[]): Promise<void> {
    if (!this.client) {
      throw new Error("Worker not started");
    }
    await this.client.replyToQuestion(requestId, answers);
  }

  async replyToPermission(requestId: string, approved: boolean): Promise<void> {
    if (!this.client) {
      throw new Error("Worker not started");
    }
    await this.client.replyToPermission(requestId, approved);
  }

  getStatus(): WorkerStatus {
    return {
      id: this.config.id,
      status: this.status,
      sessionId: this.sessionId ?? undefined,
      lastError: this.lastError,
    };
  }

  get id(): string {
    return this.config.id;
  }

  get port(): number {
    return this.config.port;
  }

  get isRunning(): boolean {
    return this.process?.isRunning ?? false;
  }
}

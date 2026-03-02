import { createParser, type ParsedEvent, type ReconnectInterval } from "eventsource-parser";
import type { StreamEvent } from "./events";
import type {
  HealthResponse,
  Session,
  MessageResponse,
  Project,
  ProviderResponse,
  OpenCodeClientOptions,
  ModelSpec,
  PermissionReply,
  QuestionInfo,
  Part,
} from "./types";

const DEFAULT_HOST = "127.0.0.1";
const DEFAULT_PORT = 14096;

export class OpenCodeClient {
  private baseUrl: string;
  private abortController: AbortController | null = null;

  constructor(options: OpenCodeClientOptions = {}) {
    if (options.baseUrl) {
      this.baseUrl = options.baseUrl.replace(/\/$/, "");
    } else {
      const host = options.host ?? DEFAULT_HOST;
      const port = options.port ?? DEFAULT_PORT;
      this.baseUrl = `http://${host}:${port}`;
    }
  }

  getBaseUrl(): string {
    return this.baseUrl;
  }

  async health(): Promise<HealthResponse> {
    const response = await fetch(`${this.baseUrl}/global/health`);
    if (!response.ok) {
      throw new Error(`Health check failed: ${response.status}`);
    }
    return response.json();
  }

  async isHealthy(): Promise<boolean> {
    try {
      const health = await this.health();
      return health.healthy;
    } catch {
      return false;
    }
  }

  async createSession(title?: string): Promise<Session> {
    const response = await fetch(`${this.baseUrl}/session`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ title }),
    });
    if (!response.ok) {
      const text = await response.text();
      throw new Error(`Failed to create session: ${response.status} - ${text}`);
    }
    return response.json();
  }

  async sendMessage(sessionId: string, text: string, model?: string): Promise<MessageResponse> {
    const modelSpec = this.parseModelString(model);

    const response = await fetch(`${this.baseUrl}/session/${sessionId}/message`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        parts: [{ type: "text", text }],
        model: modelSpec,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to send message: ${response.status} - ${errorText}`);
    }
    return response.json();
  }

  async sendMessageAsync(sessionId: string, text: string, model?: string): Promise<void> {
    const modelSpec = this.parseModelString(model);

    const response = await fetch(`${this.baseUrl}/session/${sessionId}/prompt_async`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        parts: [{ type: "text", text }],
        model: modelSpec,
      }),
    });

    if (!response.ok) {
      const errorText = await response.text();
      throw new Error(`Failed to send async message: ${response.status} - ${errorText}`);
    }
  }

  async listProjects(): Promise<Project[]> {
    const response = await fetch(`${this.baseUrl}/project`);
    if (!response.ok) {
      throw new Error(`Failed to list projects: ${response.status}`);
    }
    return response.json();
  }

  async getCurrentProject(): Promise<Project> {
    const response = await fetch(`${this.baseUrl}/project/current`);
    if (!response.ok) {
      throw new Error(`Failed to get current project: ${response.status}`);
    }
    return response.json();
  }

  async getPath(): Promise<string> {
    const response = await fetch(`${this.baseUrl}/path`);
    if (!response.ok) {
      throw new Error(`Failed to get path: ${response.status}`);
    }
    const text = await response.text();
    try {
      const parsed = JSON.parse(text);
      if (typeof parsed === "object" && parsed.path) {
        return parsed.path;
      }
      if (typeof parsed === "string") {
        return parsed;
      }
    } catch {
      // Fall through
    }
    return text.replace(/^"|"$/g, "");
  }

  async getProviders(): Promise<ProviderResponse> {
    const response = await fetch(`${this.baseUrl}/provider`);
    if (!response.ok) {
      throw new Error(`Failed to get providers: ${response.status}`);
    }
    return response.json();
  }

  async getConfig(): Promise<unknown> {
    const response = await fetch(`${this.baseUrl}/config`);
    if (!response.ok) {
      throw new Error(`Failed to get config: ${response.status}`);
    }
    return response.json();
  }

  async setModel(providerId: string, modelId: string): Promise<void> {
    const response = await fetch(`${this.baseUrl}/config`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ model: `${providerId}/${modelId}` }),
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`Failed to set model: ${response.status} - ${text}`);
    }
  }

  async replyToQuestion(requestId: string, answers: string[][]): Promise<void> {
    const response = await fetch(`${this.baseUrl}/question/${requestId}/reply`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ answers }),
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`Failed to reply to question: ${response.status} - ${text}`);
    }
  }

  async replyToPermission(requestId: string, reply: PermissionReply): Promise<void> {
    const response = await fetch(`${this.baseUrl}/permission/${requestId}/reply`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ reply }),
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`Failed to reply to permission: ${response.status} - ${text}`);
    }
  }

  subscribeEvents(callback: (event: StreamEvent) => void): () => void {
    this.abortController = new AbortController();
    const { signal } = this.abortController;

    const connect = async () => {
      try {
        const response = await fetch(`${this.baseUrl}/event`, {
          headers: { Accept: "text/event-stream" },
          signal,
        });

        if (!response.ok) {
          callback({ type: "error", message: `SSE connection failed: ${response.status}` });
          return;
        }

        callback({ type: "connected" });

        const reader = response.body?.getReader();
        if (!reader) {
          callback({ type: "error", message: "No response body" });
          return;
        }

        const decoder = new TextDecoder();
        const parser = createParser((event: ParsedEvent | ReconnectInterval) => {
          if (event.type === "event") {
            const streamEvent = this.parseSSEMessage(event.data);
            if (streamEvent) {
              callback(streamEvent);
            }
          }
        });

        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          parser.feed(decoder.decode(value, { stream: true }));
        }
      } catch (err) {
        if (signal.aborted) return;
        const message = err instanceof Error ? err.message : String(err);
        callback({ type: "error", message });
      }
    };

    connect();

    return () => {
      this.abortController?.abort();
      this.abortController = null;
    };
  }

  private parseModelString(model?: string): ModelSpec | undefined {
    if (!model) return undefined;
    const parts = model.split("/");
    if (parts.length !== 2) return undefined;
    return {
      providerID: parts[0],
      modelID: parts[1],
    };
  }

  private parseSSEMessage(data: string): StreamEvent | null {
    try {
      const parsed = JSON.parse(data);
      const eventType = parsed.type ?? "unknown";

      switch (eventType) {
        case "message.part.updated":
          return this.parsePartUpdated(parsed);
        case "session.idle":
          return this.parseSessionIdle(parsed);
        case "question.asked":
          return this.parseQuestionAsked(parsed);
        case "permission.asked":
          return this.parsePermissionAsked(parsed);
        default:
          return null;
      }
    } catch {
      return null;
    }
  }

  private parsePartUpdated(parsed: Record<string, unknown>): StreamEvent | null {
    const props = parsed.properties as Record<string, unknown> | undefined;
    const partData = props?.part as Record<string, unknown> | undefined;
    if (!partData) return null;

    const sessionId = String(partData.sessionID ?? "");
    const partType = String(partData.type ?? "");

    if (partType === "text") {
      const part: Part = {
        id: String(partData.id ?? ""),
        type: partType,
        text: partData.text as string | undefined,
      };
      if (part.text) {
        return { type: "part_updated", sessionId, part };
      }
    } else if (partType === "tool") {
      const state = partData.state as Record<string, unknown> | undefined;
      return {
        type: "tool_call",
        sessionId,
        toolName: String(partData.tool ?? "unknown"),
        status: String(state?.status ?? "unknown"),
        input: state?.input ?? null,
      };
    }
    return null;
  }

  private parseSessionIdle(parsed: Record<string, unknown>): StreamEvent | null {
    const props = parsed.properties as Record<string, unknown> | undefined;
    const sessionId = String(props?.sessionID ?? "");
    return { type: "session_idle", sessionId };
  }

  private parseQuestionAsked(parsed: Record<string, unknown>): StreamEvent | null {
    const props = parsed.properties as Record<string, unknown> | undefined;
    if (!props) return null;

    const requestId = String(props.id ?? "");
    const sessionId = String(props.sessionID ?? "");
    const questions = (props.questions as QuestionInfo[]) ?? [];

    return { type: "question_asked", sessionId, requestId, questions };
  }

  private parsePermissionAsked(parsed: Record<string, unknown>): StreamEvent | null {
    const props = parsed.properties as Record<string, unknown> | undefined;
    if (!props) return null;

    const requestId = String(props.id ?? "");
    const sessionId = String(props.sessionID ?? "");
    const permission = String(props.permission ?? "");
    const patterns = ((props.patterns as string[]) ?? []).filter(
      (p): p is string => typeof p === "string"
    );

    return { type: "permission_asked", sessionId, requestId, permission, patterns };
  }
}

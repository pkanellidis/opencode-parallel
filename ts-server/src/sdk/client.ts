import EventSource from "eventsource";
import type {
  HealthResponse,
  Session,
  MessageResponse,
  Project,
  Provider,
  ConfigResponse,
  ConfigUpdateRequest,
  CreateSessionRequest,
  SendMessageRequest,
  StreamEvent,
  QuestionInfo,
} from "../types/index.js";

export class OpenCodeClient {
  private baseUrl: string;

  constructor(port: number, host: string = "127.0.0.1") {
    this.baseUrl = `http://${host}:${port}`;
  }

  private async request<T>(
    path: string,
    options: RequestInit = {}
  ): Promise<T> {
    const url = `${this.baseUrl}${path}`;
    const response = await fetch(url, {
      ...options,
      headers: {
        "Content-Type": "application/json",
        ...options.headers,
      },
    });

    if (!response.ok) {
      const text = await response.text();
      throw new Error(`Request failed: ${response.status} ${text}`);
    }

    const contentType = response.headers.get("content-type");
    if (contentType?.includes("application/json")) {
      return response.json() as Promise<T>;
    }

    return response.text() as unknown as T;
  }

  async health(): Promise<HealthResponse> {
    return this.request<HealthResponse>("/global/health");
  }

  async waitForHealth(timeoutMs: number = 30000): Promise<boolean> {
    const start = Date.now();
    while (Date.now() - start < timeoutMs) {
      try {
        await this.health();
        return true;
      } catch {
        await new Promise((resolve) => setTimeout(resolve, 100));
      }
    }
    return false;
  }

  async createSession(options?: CreateSessionRequest): Promise<Session> {
    return this.request<Session>("/session", {
      method: "POST",
      body: JSON.stringify(options ?? {}),
    });
  }

  async sendMessage(
    sessionId: string,
    request: SendMessageRequest
  ): Promise<MessageResponse> {
    const body: Record<string, unknown> = {
      content: [{ type: "text", text: request.content }],
    };

    if (request.model) {
      body.model = request.model;
    }

    return this.request<MessageResponse>(`/session/${sessionId}/message`, {
      method: "POST",
      body: JSON.stringify(body),
    });
  }

  async sendMessageAsync(
    sessionId: string,
    request: SendMessageRequest,
    onEvent: (event: StreamEvent) => void
  ): Promise<void> {
    const body: Record<string, unknown> = {
      content: [{ type: "text", text: request.content }],
    };

    if (request.model) {
      body.model = request.model;
    }

    return new Promise((resolve, reject) => {
      const url = `${this.baseUrl}/session/${sessionId}/prompt_async`;

      fetch(url, {
        method: "POST",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(body),
      })
        .then(async (response) => {
          if (!response.ok) {
            throw new Error(`Request failed: ${response.status}`);
          }

          if (!response.body) {
            throw new Error("No response body");
          }

          const reader = response.body.getReader();
          const decoder = new TextDecoder();
          let buffer = "";

          while (true) {
            const { done, value } = await reader.read();
            if (done) break;

            buffer += decoder.decode(value, { stream: true });
            const lines = buffer.split("\n");
            buffer = lines.pop() ?? "";

            for (const line of lines) {
              if (line.startsWith("data: ")) {
                const data = line.slice(6);
                if (data === "[DONE]") {
                  resolve();
                  return;
                }
                try {
                  const event = this.parseStreamEvent(JSON.parse(data));
                  if (event) onEvent(event);
                } catch {
                  // Skip invalid JSON
                }
              }
            }
          }

          resolve();
        })
        .catch(reject);
    });
  }

  private parseStreamEvent(data: Record<string, unknown>): StreamEvent | null {
    const type = data.type as string;

    switch (type) {
      case "part.updated":
        return {
          type: "part.updated",
          sessionId: data.sessionId as string,
          part: data.part as StreamEvent extends { part: infer P } ? P : never,
        } as StreamEvent;

      case "tool.call":
        return {
          type: "tool.call",
          sessionId: data.sessionId as string,
          toolName: data.toolName as string,
          status: data.status as string,
          input: data.input,
        };

      case "session.idle":
        return {
          type: "session.idle",
          sessionId: data.sessionId as string,
        };

      case "question.asked":
        return {
          type: "question.asked",
          sessionId: data.sessionId as string,
          requestId: data.requestId as string,
          questions: data.questions as QuestionInfo[],
        };

      case "permission.asked":
        return {
          type: "permission.asked",
          sessionId: data.sessionId as string,
          requestId: data.requestId as string,
          permission: data.permission as string,
          patterns: data.patterns as string[],
        };

      default:
        return null;
    }
  }

  async getProjects(): Promise<Project[]> {
    return this.request<Project[]>("/project");
  }

  async getCurrentProject(): Promise<Project> {
    return this.request<Project>("/project/current");
  }

  async getPath(): Promise<string> {
    return this.request<string>("/path");
  }

  async getProviders(): Promise<Provider[]> {
    return this.request<Provider[]>("/provider");
  }

  async getConfig(): Promise<ConfigResponse> {
    return this.request<ConfigResponse>("/config");
  }

  async updateConfig(config: ConfigUpdateRequest): Promise<void> {
    await this.request<void>("/config", {
      method: "PATCH",
      body: JSON.stringify(config),
    });
  }

  async setModel(providerId: string, modelId: string): Promise<void> {
    await this.updateConfig({
      model: `${providerId}/${modelId}`,
    });
  }

  async replyToQuestion(requestId: string, answers: string[]): Promise<void> {
    await this.request<void>(`/question/${requestId}/reply`, {
      method: "POST",
      body: JSON.stringify({ answers }),
    });
  }

  async replyToPermission(
    requestId: string,
    approved: boolean
  ): Promise<void> {
    await this.request<void>(`/permission/${requestId}/reply`, {
      method: "POST",
      body: JSON.stringify({ approved }),
    });
  }

  subscribeToEvents(onEvent: (event: StreamEvent) => void): () => void {
    const eventSource = new EventSource(`${this.baseUrl}/event`);

    eventSource.onopen = () => {
      onEvent({ type: "connected" });
    };

    eventSource.onmessage = (e: MessageEvent) => {
      try {
        const data = JSON.parse(e.data as string);
        const event = this.parseStreamEvent(data);
        if (event) onEvent(event);
      } catch {
        // Skip invalid events
      }
    };

    eventSource.onerror = () => {
      onEvent({ type: "error", message: "EventSource connection error" });
    };

    return () => eventSource.close();
  }
}

export interface HealthResponse {
  healthy: boolean;
  version: string;
}

export interface Session {
  id: string;
  title?: string;
  parentID?: string;
}

export interface Message {
  id: string;
  sessionID: string;
  role: string;
}

export interface Part {
  id: string;
  type: string;
  text?: string;
}

export interface MessageResponse {
  info: Message;
  parts: Part[];
}

export interface Project {
  id: string;
  worktree: string;
  vcs?: string;
}

export interface Model {
  id: string;
  name?: string;
}

export interface Provider {
  id: string;
  name?: string;
  models: Record<string, Model>;
}

export interface ProviderResponse {
  all: Provider[];
  default: unknown;
  connected: string[];
}

export interface QuestionOption {
  label: string;
  description?: string;
}

export interface QuestionInfo {
  question: string;
  header?: string;
  options: QuestionOption[];
}

export interface ModelSpec {
  providerID: string;
  modelID: string;
}

export interface MessagePart {
  type: string;
  text: string;
}

export interface SendMessageRequest {
  parts: MessagePart[];
  model?: ModelSpec;
}

export interface CreateSessionRequest {
  title?: string;
}

export type PermissionReply = "once" | "always" | "reject";

export interface OpenCodeClientOptions {
  baseUrl?: string;
  port?: number;
  host?: string;
}

export interface HealthResponse {
  version: string;
  status: string;
}

export interface Session {
  id: string;
  title: string;
  parentID?: string;
}

export interface Part {
  type: string;
  content?: string;
  toolName?: string;
  toolInput?: unknown;
  toolOutput?: string;
}

export interface Message {
  id: string;
  sessionID: string;
  role: "user" | "assistant" | "system";
  parts: Part[];
}

export interface MessageResponse {
  id: string;
  sessionID: string;
  role: string;
  parts: Part[];
}

export interface Project {
  name: string;
  worktree: string;
  vcs?: {
    type: string;
    branch?: string;
  };
}

export interface Model {
  id: string;
  name: string;
  provider: string;
}

export interface Provider {
  id: string;
  name: string;
  models: Model[];
}

export interface QuestionOption {
  label: string;
  description?: string;
}

export interface QuestionInfo {
  question: string;
  header?: string;
  options: QuestionOption[];
  multiple?: boolean;
}

export interface ModelSpec {
  providerID: string;
  modelID: string;
}

export interface CreateSessionRequest {
  title?: string;
  parentID?: string;
}

export interface SendMessageRequest {
  content: string;
  model?: ModelSpec;
}

export interface ConfigResponse {
  model?: string;
  [key: string]: unknown;
}

export interface ConfigUpdateRequest {
  model?: string;
  [key: string]: unknown;
}

export type StreamEventType =
  | "connected"
  | "part.updated"
  | "tool.call"
  | "session.idle"
  | "question.asked"
  | "permission.asked"
  | "error";

export interface StreamEventConnected {
  type: "connected";
}

export interface StreamEventPartUpdated {
  type: "part.updated";
  sessionId: string;
  part: Part;
}

export interface StreamEventToolCall {
  type: "tool.call";
  sessionId: string;
  toolName: string;
  status: string;
  input: unknown;
}

export interface StreamEventSessionIdle {
  type: "session.idle";
  sessionId: string;
}

export interface StreamEventQuestionAsked {
  type: "question.asked";
  sessionId: string;
  requestId: string;
  questions: QuestionInfo[];
}

export interface StreamEventPermissionAsked {
  type: "permission.asked";
  sessionId: string;
  requestId: string;
  permission: string;
  patterns: string[];
}

export interface StreamEventError {
  type: "error";
  message: string;
}

export type StreamEvent =
  | StreamEventConnected
  | StreamEventPartUpdated
  | StreamEventToolCall
  | StreamEventSessionIdle
  | StreamEventQuestionAsked
  | StreamEventPermissionAsked
  | StreamEventError;

export interface WorkerConfig {
  id: string;
  port: number;
  workdir?: string;
  model?: ModelSpec;
}

export interface WorkerStatus {
  id: string;
  status: "idle" | "running" | "error" | "stopped";
  sessionId?: string;
  lastError?: string;
}

export interface TaskRequest {
  workerId: string;
  prompt: string;
  model?: ModelSpec;
}

export interface TaskResult {
  workerId: string;
  success: boolean;
  response?: MessageResponse;
  error?: string;
}

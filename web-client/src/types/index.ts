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

export interface PathResponse {
  path: string;
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

export type StreamEventType =
  | 'connected'
  | 'part_updated'
  | 'tool_call'
  | 'session_idle'
  | 'question_asked'
  | 'permission_asked'
  | 'error';

export interface StreamEventPartUpdated {
  type: 'part_updated';
  sessionId: string;
  part: Part;
}

export interface EditToolInput {
  filePath: string;
  oldString: string;
  newString: string;
  replaceAll?: boolean;
}

export interface WriteToolInput {
  filePath: string;
  content: string;
}

export interface ReadToolInput {
  filePath: string;
  offset?: number;
  limit?: number;
}

export interface BashToolInput {
  command: string;
  workdir?: string;
  description?: string;
  timeout?: number;
}

export interface GlobToolInput {
  pattern: string;
  path?: string;
}

export interface GrepToolInput {
  pattern: string;
  path?: string;
  include?: string;
}

export type ToolInput =
  | EditToolInput
  | WriteToolInput
  | ReadToolInput
  | BashToolInput
  | GlobToolInput
  | GrepToolInput
  | Record<string, unknown>;

export interface ToolCallDetails {
  id: string;
  toolName: string;
  status: 'running' | 'completed' | 'error';
  input: ToolInput;
  output?: string;
  error?: string;
  startTime: number;
  endTime?: number;
}

export interface StreamEventToolCall {
  type: 'tool_call';
  sessionId: string;
  toolName: string;
  status: string;
  input: unknown;
  toolCallId?: string;
  output?: string;
  error?: string;
}

export interface StreamEventSessionIdle {
  type: 'session_idle';
  sessionId: string;
}

export interface StreamEventQuestionAsked {
  type: 'question_asked';
  sessionId: string;
  requestId: string;
  questions: QuestionInfo[];
}

export interface StreamEventPermissionAsked {
  type: 'permission_asked';
  sessionId: string;
  requestId: string;
  permission: string;
  patterns: string[];
}

export interface StreamEventConnected {
  type: 'connected';
}

export interface StreamEventError {
  type: 'error';
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

export type WorkerState = 'idle' | 'starting' | 'running' | 'waiting_input' | 'complete' | 'error';

export interface Worker {
  id: number;
  description: string;
  sessionId?: string;
  state: WorkerState;
  output: string[];
  streamingContent: string;
  currentTool?: string;
  toolHistory: string[];
  toolCalls: ToolCallDetails[];
  pendingQuestion?: QuestionInfo;
  pendingQuestionRequestId?: string;
  pendingPermission?: { permission: string; patterns: string[] };
  pendingPermissionRequestId?: string;
}

export interface UISession {
  id: number;
  name: string;
  messages: Array<{ content: string; isUser: boolean }>;
  workers: Worker[];
  orchestratorSessionId?: string;
  originalRequest?: string;
}

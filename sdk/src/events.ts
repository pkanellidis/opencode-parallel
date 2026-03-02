import type { Part, QuestionInfo } from "./types";

export type StreamEvent =
  | { type: "connected" }
  | { type: "part_updated"; sessionId: string; part: Part }
  | {
      type: "tool_call";
      sessionId: string;
      toolName: string;
      status: string;
      input: unknown;
    }
  | { type: "session_idle"; sessionId: string }
  | {
      type: "question_asked";
      sessionId: string;
      requestId: string;
      questions: QuestionInfo[];
    }
  | {
      type: "permission_asked";
      sessionId: string;
      requestId: string;
      permission: string;
      patterns: string[];
    }
  | { type: "error"; message: string };

export function getSessionId(event: StreamEvent): string | undefined {
  switch (event.type) {
    case "part_updated":
    case "tool_call":
    case "session_idle":
    case "question_asked":
    case "permission_asked":
      return event.sessionId;
    default:
      return undefined;
  }
}

export function isErrorEvent(event: StreamEvent): event is { type: "error"; message: string } {
  return event.type === "error";
}

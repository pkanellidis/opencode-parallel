import { describe, it, expect } from "vitest";
import { getSessionId, isErrorEvent, type StreamEvent } from "./events";

describe("getSessionId", () => {
  it("returns sessionId for part_updated event", () => {
    const event: StreamEvent = {
      type: "part_updated",
      sessionId: "ses_123",
      part: { id: "p1", type: "text", text: "hello" },
    };
    expect(getSessionId(event)).toBe("ses_123");
  });

  it("returns sessionId for tool_call event", () => {
    const event: StreamEvent = {
      type: "tool_call",
      sessionId: "ses_456",
      toolName: "bash",
      status: "running",
      input: {},
    };
    expect(getSessionId(event)).toBe("ses_456");
  });

  it("returns sessionId for session_idle event", () => {
    const event: StreamEvent = {
      type: "session_idle",
      sessionId: "ses_789",
    };
    expect(getSessionId(event)).toBe("ses_789");
  });

  it("returns sessionId for question_asked event", () => {
    const event: StreamEvent = {
      type: "question_asked",
      sessionId: "ses_abc",
      requestId: "req_1",
      questions: [],
    };
    expect(getSessionId(event)).toBe("ses_abc");
  });

  it("returns sessionId for permission_asked event", () => {
    const event: StreamEvent = {
      type: "permission_asked",
      sessionId: "ses_def",
      requestId: "req_2",
      permission: "write",
      patterns: [],
    };
    expect(getSessionId(event)).toBe("ses_def");
  });

  it("returns undefined for connected event", () => {
    const event: StreamEvent = { type: "connected" };
    expect(getSessionId(event)).toBeUndefined();
  });

  it("returns undefined for error event", () => {
    const event: StreamEvent = { type: "error", message: "test error" };
    expect(getSessionId(event)).toBeUndefined();
  });
});

describe("isErrorEvent", () => {
  it("returns true for error event", () => {
    const event: StreamEvent = { type: "error", message: "test" };
    expect(isErrorEvent(event)).toBe(true);
  });

  it("returns false for connected event", () => {
    const event: StreamEvent = { type: "connected" };
    expect(isErrorEvent(event)).toBe(false);
  });

  it("returns false for session_idle event", () => {
    const event: StreamEvent = { type: "session_idle", sessionId: "ses_1" };
    expect(isErrorEvent(event)).toBe(false);
  });
});

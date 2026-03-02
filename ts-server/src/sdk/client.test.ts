import { describe, it, expect, vi, beforeEach } from "vitest";
import { OpenCodeClient } from "./client.js";

describe("OpenCodeClient", () => {
  let client: OpenCodeClient;

  beforeEach(() => {
    client = new OpenCodeClient(4000);
    vi.resetAllMocks();
  });

  describe("constructor", () => {
    it("should create client with default host", () => {
      const c = new OpenCodeClient(4000);
      expect(c).toBeInstanceOf(OpenCodeClient);
    });

    it("should create client with custom host", () => {
      const c = new OpenCodeClient(4000, "192.168.1.1");
      expect(c).toBeInstanceOf(OpenCodeClient);
    });
  });

  describe("health", () => {
    it("should call health endpoint", async () => {
      const mockResponse = { version: "1.0.0", status: "ok" };
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        headers: new Headers({ "content-type": "application/json" }),
        json: () => Promise.resolve(mockResponse),
      });

      const result = await client.health();
      expect(result).toEqual(mockResponse);
      expect(fetch).toHaveBeenCalledWith(
        "http://127.0.0.1:4000/global/health",
        expect.objectContaining({ headers: expect.any(Object) })
      );
    });
  });

  describe("createSession", () => {
    it("should create a session", async () => {
      const mockSession = { id: "session-123", title: "Test" };
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        headers: new Headers({ "content-type": "application/json" }),
        json: () => Promise.resolve(mockSession),
      });

      const result = await client.createSession();
      expect(result).toEqual(mockSession);
      expect(fetch).toHaveBeenCalledWith(
        "http://127.0.0.1:4000/session",
        expect.objectContaining({ method: "POST" })
      );
    });
  });

  describe("sendMessage", () => {
    it("should send message to session", async () => {
      const mockResponse = {
        id: "msg-1",
        sessionID: "session-123",
        role: "assistant",
        parts: [{ type: "text", content: "Hello" }],
      };
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        headers: new Headers({ "content-type": "application/json" }),
        json: () => Promise.resolve(mockResponse),
      });

      const result = await client.sendMessage("session-123", {
        content: "Hello",
      });
      expect(result).toEqual(mockResponse);
    });

    it("should include model spec when provided", async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        headers: new Headers({ "content-type": "application/json" }),
        json: () => Promise.resolve({}),
      });

      await client.sendMessage("session-123", {
        content: "Hello",
        model: { providerID: "openai", modelID: "gpt-4" },
      });

      expect(fetch).toHaveBeenCalledWith(
        "http://127.0.0.1:4000/session/session-123/message",
        expect.objectContaining({
          body: expect.stringContaining('"model"'),
        })
      );
    });
  });

  describe("setModel", () => {
    it("should update config with model", async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        headers: new Headers({ "content-type": "application/json" }),
        json: () => Promise.resolve({}),
      });

      await client.setModel("anthropic", "claude-3");

      expect(fetch).toHaveBeenCalledWith(
        "http://127.0.0.1:4000/config",
        expect.objectContaining({
          method: "PATCH",
          body: JSON.stringify({ model: "anthropic/claude-3" }),
        })
      );
    });
  });

  describe("waitForHealth", () => {
    it("should return true when server is healthy", async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        headers: new Headers({ "content-type": "application/json" }),
        json: () => Promise.resolve({ status: "ok" }),
      });

      const result = await client.waitForHealth(1000);
      expect(result).toBe(true);
    });

    it("should return false after timeout", async () => {
      global.fetch = vi.fn().mockRejectedValue(new Error("Connection refused"));

      const result = await client.waitForHealth(100);
      expect(result).toBe(false);
    });
  });
});

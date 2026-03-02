import { describe, it, expect } from "vitest";
import { OpenCodeClient } from "./client";

describe("OpenCodeClient", () => {
  describe("constructor", () => {
    it("uses default host and port when no options provided", () => {
      const client = new OpenCodeClient();
      expect(client.getBaseUrl()).toBe("http://127.0.0.1:14096");
    });

    it("uses custom port", () => {
      const client = new OpenCodeClient({ port: 8080 });
      expect(client.getBaseUrl()).toBe("http://127.0.0.1:8080");
    });

    it("uses custom host and port", () => {
      const client = new OpenCodeClient({ host: "localhost", port: 3000 });
      expect(client.getBaseUrl()).toBe("http://localhost:3000");
    });

    it("uses baseUrl directly when provided", () => {
      const client = new OpenCodeClient({ baseUrl: "https://api.example.com" });
      expect(client.getBaseUrl()).toBe("https://api.example.com");
    });

    it("strips trailing slash from baseUrl", () => {
      const client = new OpenCodeClient({ baseUrl: "https://api.example.com/" });
      expect(client.getBaseUrl()).toBe("https://api.example.com");
    });

    it("baseUrl takes precedence over host/port", () => {
      const client = new OpenCodeClient({
        baseUrl: "https://api.example.com",
        host: "ignored",
        port: 9999,
      });
      expect(client.getBaseUrl()).toBe("https://api.example.com");
    });
  });
});

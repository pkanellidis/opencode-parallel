import { describe, it, expect, beforeEach } from "vitest";
import { Orchestrator } from "./orchestrator.js";

describe("Orchestrator", () => {
  let orchestrator: Orchestrator;

  beforeEach(() => {
    orchestrator = new Orchestrator({
      basePort: 5000,
      maxWorkers: 3,
      workdir: "/tmp",
    });
  });

  describe("createWorker", () => {
    it("should create a worker with unique id", async () => {
      const worker = await orchestrator.createWorker("worker-1");
      expect(worker.id).toBe("worker-1");
      expect(worker.port).toBe(5000);
    });

    it("should increment port for each worker", async () => {
      const worker1 = await orchestrator.createWorker("worker-1");
      const worker2 = await orchestrator.createWorker("worker-2");

      expect(worker1.port).toBe(5000);
      expect(worker2.port).toBe(5001);
    });

    it("should throw when max workers reached", async () => {
      await orchestrator.createWorker("w1");
      await orchestrator.createWorker("w2");
      await orchestrator.createWorker("w3");

      await expect(orchestrator.createWorker("w4")).rejects.toThrow(
        "Maximum workers (3) reached"
      );
    });

    it("should throw when worker id already exists", async () => {
      await orchestrator.createWorker("worker-1");

      await expect(orchestrator.createWorker("worker-1")).rejects.toThrow(
        "Worker with id 'worker-1' already exists"
      );
    });
  });

  describe("getWorker", () => {
    it("should return worker by id", async () => {
      await orchestrator.createWorker("test-worker");
      const worker = orchestrator.getWorker("test-worker");

      expect(worker).toBeDefined();
      expect(worker?.id).toBe("test-worker");
    });

    it("should return undefined for non-existent worker", () => {
      const worker = orchestrator.getWorker("non-existent");
      expect(worker).toBeUndefined();
    });
  });

  describe("getAllWorkers", () => {
    it("should return all workers", async () => {
      await orchestrator.createWorker("w1");
      await orchestrator.createWorker("w2");

      const workers = orchestrator.getAllWorkers();
      expect(workers).toHaveLength(2);
    });

    it("should return empty array when no workers", () => {
      const workers = orchestrator.getAllWorkers();
      expect(workers).toHaveLength(0);
    });
  });

  describe("getWorkerStatuses", () => {
    it("should return statuses for all workers", async () => {
      await orchestrator.createWorker("w1");
      await orchestrator.createWorker("w2");

      const statuses = orchestrator.getWorkerStatuses();
      expect(statuses).toHaveLength(2);
      expect(statuses[0].status).toBe("stopped");
      expect(statuses[1].status).toBe("stopped");
    });
  });

  describe("removeWorker", () => {
    it("should remove a worker", async () => {
      await orchestrator.createWorker("worker-1");
      await orchestrator.removeWorker("worker-1");

      expect(orchestrator.getWorker("worker-1")).toBeUndefined();
    });

    it("should handle removing non-existent worker", async () => {
      await expect(
        orchestrator.removeWorker("non-existent")
      ).resolves.not.toThrow();
    });
  });

  describe("executeTask", () => {
    it("should return error for non-existent worker", async () => {
      const result = await orchestrator.executeTask({
        workerId: "non-existent",
        prompt: "test",
      });

      expect(result.success).toBe(false);
      expect(result.error).toContain("not found");
    });
  });

  describe("shutdown", () => {
    it("should stop all workers and clear map", async () => {
      await orchestrator.createWorker("w1");
      await orchestrator.createWorker("w2");

      await orchestrator.shutdown();

      expect(orchestrator.getAllWorkers()).toHaveLength(0);
    });
  });
});

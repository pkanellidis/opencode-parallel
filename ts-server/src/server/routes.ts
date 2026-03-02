import { Router, Request, Response } from "express";
import asyncHandler from "express-async-handler";
import { Orchestrator } from "./orchestrator.js";
import type { TaskRequest, ModelSpec } from "../types/index.js";

export function createRoutes(orchestrator: Orchestrator): Router {
  const router = Router();

  router.get(
    "/health",
    asyncHandler(async (_req: Request, res: Response) => {
      res.json({ status: "ok", version: "1.0.0" });
    })
  );

  router.get(
    "/workers",
    asyncHandler(async (_req: Request, res: Response) => {
      const statuses = orchestrator.getWorkerStatuses();
      res.json(statuses);
    })
  );

  router.post(
    "/workers",
    asyncHandler(async (req: Request, res: Response) => {
      const { id, workdir } = req.body as { id: string; workdir?: string };

      if (!id) {
        res.status(400).json({ error: "Worker id is required" });
        return;
      }

      const worker = await orchestrator.createWorker(id, workdir);
      await worker.start();
      await worker.createSession();

      res.status(201).json(worker.getStatus());
    })
  );

  router.get(
    "/workers/:id",
    asyncHandler(async (req: Request, res: Response) => {
      const worker = orchestrator.getWorker(req.params.id);

      if (!worker) {
        res.status(404).json({ error: "Worker not found" });
        return;
      }

      res.json(worker.getStatus());
    })
  );

  router.delete(
    "/workers/:id",
    asyncHandler(async (req: Request, res: Response) => {
      await orchestrator.removeWorker(req.params.id);
      res.status(204).send();
    })
  );

  router.post(
    "/workers/:id/start",
    asyncHandler(async (req: Request, res: Response) => {
      await orchestrator.startWorker(req.params.id);
      const worker = orchestrator.getWorker(req.params.id);
      res.json(worker?.getStatus());
    })
  );

  router.post(
    "/workers/:id/stop",
    asyncHandler(async (req: Request, res: Response) => {
      await orchestrator.stopWorker(req.params.id);
      const worker = orchestrator.getWorker(req.params.id);
      res.json(worker?.getStatus());
    })
  );

  router.post(
    "/workers/:id/session",
    asyncHandler(async (req: Request, res: Response) => {
      const worker = orchestrator.getWorker(req.params.id);

      if (!worker) {
        res.status(404).json({ error: "Worker not found" });
        return;
      }

      const sessionId = await worker.createSession();
      res.json({ sessionId });
    })
  );

  router.post(
    "/workers/:id/message",
    asyncHandler(async (req: Request, res: Response) => {
      const worker = orchestrator.getWorker(req.params.id);

      if (!worker) {
        res.status(404).json({ error: "Worker not found" });
        return;
      }

      const { content, model } = req.body as {
        content: string;
        model?: ModelSpec;
      };

      if (!content) {
        res.status(400).json({ error: "Content is required" });
        return;
      }

      const response = await worker.sendMessage({ content, model });
      res.json(response);
    })
  );

  router.post(
    "/workers/:id/message/stream",
    asyncHandler(async (req: Request, res: Response) => {
      const worker = orchestrator.getWorker(req.params.id);

      if (!worker) {
        res.status(404).json({ error: "Worker not found" });
        return;
      }

      const { content, model } = req.body as {
        content: string;
        model?: ModelSpec;
      };

      if (!content) {
        res.status(400).json({ error: "Content is required" });
        return;
      }

      res.setHeader("Content-Type", "text/event-stream");
      res.setHeader("Cache-Control", "no-cache");
      res.setHeader("Connection", "keep-alive");

      await worker.sendMessageAsync({ content, model }, (event) => {
        res.write(`data: ${JSON.stringify(event)}\n\n`);
      });

      res.write("data: [DONE]\n\n");
      res.end();
    })
  );

  router.post(
    "/workers/:id/question/:requestId/reply",
    asyncHandler(async (req: Request, res: Response) => {
      const worker = orchestrator.getWorker(req.params.id);

      if (!worker) {
        res.status(404).json({ error: "Worker not found" });
        return;
      }

      const { answers } = req.body as { answers: string[] };
      await worker.replyToQuestion(req.params.requestId, answers);
      res.status(204).send();
    })
  );

  router.post(
    "/workers/:id/permission/:requestId/reply",
    asyncHandler(async (req: Request, res: Response) => {
      const worker = orchestrator.getWorker(req.params.id);

      if (!worker) {
        res.status(404).json({ error: "Worker not found" });
        return;
      }

      const { approved } = req.body as { approved: boolean };
      await worker.replyToPermission(req.params.requestId, approved);
      res.status(204).send();
    })
  );

  router.post(
    "/tasks",
    asyncHandler(async (req: Request, res: Response) => {
      const task = req.body as TaskRequest;

      if (!task.workerId || !task.prompt) {
        res.status(400).json({ error: "workerId and prompt are required" });
        return;
      }

      const result = await orchestrator.executeTask(task);
      res.json(result);
    })
  );

  router.post(
    "/tasks/parallel",
    asyncHandler(async (req: Request, res: Response) => {
      const { tasks } = req.body as { tasks: TaskRequest[] };

      if (!tasks || !Array.isArray(tasks)) {
        res.status(400).json({ error: "tasks array is required" });
        return;
      }

      const results = await orchestrator.executeTasksParallel(tasks);
      res.json(results);
    })
  );

  router.post(
    "/tasks/stream",
    asyncHandler(async (req: Request, res: Response) => {
      const task = req.body as TaskRequest;

      if (!task.workerId || !task.prompt) {
        res.status(400).json({ error: "workerId and prompt are required" });
        return;
      }

      res.setHeader("Content-Type", "text/event-stream");
      res.setHeader("Cache-Control", "no-cache");
      res.setHeader("Connection", "keep-alive");

      const result = await orchestrator.executeTaskAsync(
        task,
        (workerId, event) => {
          res.write(`data: ${JSON.stringify({ workerId, event })}\n\n`);
        }
      );

      res.write(`data: ${JSON.stringify({ type: "result", result })}\n\n`);
      res.write("data: [DONE]\n\n");
      res.end();
    })
  );

  router.post(
    "/shutdown",
    asyncHandler(async (_req: Request, res: Response) => {
      res.json({ status: "shutting_down" });
      await orchestrator.shutdown();
      process.exit(0);
    })
  );

  return router;
}

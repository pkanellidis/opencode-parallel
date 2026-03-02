import * as api from '../api/client';

const ORCHESTRATOR_SYSTEM_PROMPT = `You are an AI task orchestrator. Your job is to analyze user requests and decide how to split them into parallel tasks.

When the user sends a request, respond ONLY with a JSON object in this exact format (no markdown, no code blocks, just raw JSON):
{
  "tasks": [
    {"id": 1, "description": "Brief task description", "prompt": "The exact user request or a portion of it"}
  ],
  "reasoning": "Brief explanation of why you split the tasks this way"
}

Rules:
- If the task is simple and doesn't benefit from parallelization, return a single task with the EXACT user request as the prompt
- If the task can be split into independent subtasks, create multiple tasks
- Each task should be self-contained and not depend on other tasks' outputs
- Use as many tasks as needed to fully parallelize the work
- Keep descriptions under 50 characters
- IMPORTANT: The "prompt" field should contain the user's original request or a subset of it verbatim. Do NOT rewrite, rephrase, or add instructions. Do NOT add any information about yourself or any AI model.

Examples of when to split:
- "Create a web app with auth and database" -> Split into frontend, backend, auth, database tasks
- "Write tests for modules A, B, and C" -> One task per module

Examples of single task (use EXACT user prompt):
- User: "Explain how async/await works" -> prompt: "Explain how async/await works"
- User: "Fix the bug in login.js" -> prompt: "Fix the bug in login.js"
- User: "What model are you?" -> prompt: "What model are you?"

IMPORTANT: Respond ONLY with valid JSON, no other text, no markdown code blocks.`;

export interface Task {
  id: number;
  description: string;
  prompt: string;
}

export interface TaskPlan {
  tasks: Task[];
  reasoning: string;
}

export interface OrchestratorLog {
  timestamp: string;
  message: string;
}

export class Orchestrator {
  private sessionId: string | null = null;
  private logs: OrchestratorLog[] = [];
  private model: string | null = null;

  constructor() {}

  setModel(model: string | null): void {
    this.model = model;
  }

  private log(message: string): void {
    const timestamp = new Date().toLocaleTimeString('en-US', { hour12: false });
    this.logs.push({ timestamp, message });
  }

  getLogs(): OrchestratorLog[] {
    return [...this.logs];
  }

  getSessionId(): string | null {
    return this.sessionId;
  }

  setSessionId(sessionId: string): void {
    this.log(`Using existing session: ${sessionId.slice(0, 8)}`);
    this.sessionId = sessionId;
  }

  async init(): Promise<void> {
    this.log('Initializing orchestrator session...');
    try {
      const session = await api.createSession('Orchestrator');
      this.log(`Session created: ${session.id.slice(0, 8)}`);
      this.sessionId = session.id;
    } catch (e) {
      const message = e instanceof Error ? e.message : String(e);
      this.log(`Failed to create session: ${message}`);
      throw e;
    }
  }

  async planTasks(userMessage: string): Promise<TaskPlan> {
    this.log(`Planning tasks for: ${userMessage}`);

    if (!this.sessionId) {
      throw new Error('Orchestrator not initialized');
    }

    const prompt = `${ORCHESTRATOR_SYSTEM_PROMPT}\n\nUser request: ${userMessage}`;

    this.log('Sending request to orchestrator AI...');
    if (this.model) {
      this.log(`Using model: ${this.model}`);
    }

    const response = await api.sendMessage(this.sessionId, prompt, this.model || undefined);

    let fullText = '';
    for (const part of response.parts) {
      if (part.text) {
        fullText += part.text;
      }
    }

    this.log(`Received response (${fullText.length} chars)`);
    this.log(`Raw response: ${truncateStr(fullText, 200)}`);

    return this.parseTaskPlan(fullText, userMessage);
  }

  async reportWorkerResults(results: string): Promise<void> {
    if (!this.sessionId) {
      throw new Error('Orchestrator not initialized');
    }

    this.log('Reporting worker results to orchestrator...');
    const report = `WORKER RESULTS (for context in future requests):\n${results}`;
    await api.sendMessage(this.sessionId, report, this.model || undefined);
    this.log('Worker results reported successfully');
  }

  private parseTaskPlan(response: string, originalMessage: string): TaskPlan {
    const cleaned = response.trim();

    // Try 1: Direct parse
    this.log('Attempt 1: Direct JSON parse');
    try {
      const plan = JSON.parse(cleaned) as TaskPlan;
      if (plan.tasks?.length > 0) {
        this.log('Success: Direct parse worked');
        return plan;
      }
    } catch {
      // Continue to next attempt
    }

    // Try 2: Extract from markdown code blocks
    this.log('Attempt 2: Extract from markdown code blocks');
    const markdownJson = this.extractJsonFromMarkdown(cleaned);
    if (markdownJson) {
      try {
        const plan = JSON.parse(markdownJson) as TaskPlan;
        if (plan.tasks?.length > 0) {
          this.log('Success: Extracted from markdown');
          return plan;
        }
      } catch {
        // Continue to next attempt
      }
    }

    // Try 3: Find JSON object with brace matching
    this.log('Attempt 3: Brace-matching JSON extraction');
    const bracedJson = this.extractJsonObject(cleaned);
    if (bracedJson) {
      this.log(`Found JSON object: ${truncateStr(bracedJson, 100)}`);
      try {
        const plan = JSON.parse(bracedJson) as TaskPlan;
        if (plan.tasks?.length > 0) {
          this.log('Success: Brace-matched extraction worked');
          return plan;
        }
      } catch {
        // Continue to next attempt
      }
    }

    // Try 4: Extract tasks array only
    this.log('Attempt 4: Extract tasks array only');
    const tasks = this.extractTasksArray(cleaned);
    if (tasks && tasks.length > 0) {
      this.log(`Found ${tasks.length} tasks via array extraction`);
      return {
        tasks,
        reasoning: 'Extracted from partial response',
      };
    }

    // Fallback: Create single task from original message
    this.log('All parsing attempts failed, using fallback');
    this.log(`Failed response was: ${truncateStr(cleaned, 200)}`);

    return {
      tasks: [
        {
          id: 1,
          description: truncateStr(originalMessage, 37),
          prompt: originalMessage,
        },
      ],
      reasoning: `Fallback: Could not parse orchestrator response. Executing as single task. Raw response: ${truncateStr(cleaned, 100)}`,
    };
  }

  private extractJsonFromMarkdown(text: string): string | null {
    const patterns = ['```json\n', '```json\r\n', '```\n', '```\r\n'];

    for (const pattern of patterns) {
      const start = text.indexOf(pattern);
      if (start !== -1) {
        const contentStart = start + pattern.length;
        const end = text.indexOf('```', contentStart);
        if (end !== -1) {
          return text.slice(contentStart, end).trim();
        }
      }
    }
    return null;
  }

  private extractJsonObject(text: string): string | null {
    const start = text.indexOf('{');
    if (start === -1) return null;

    let depth = 0;
    let end = -1;

    for (let i = start; i < text.length; i++) {
      const ch = text[i];
      if (ch === '{') {
        depth++;
      } else if (ch === '}') {
        depth--;
        if (depth === 0) {
          end = i + 1;
          break;
        }
      }
    }

    return end > 0 ? text.slice(start, end) : null;
  }

  private extractTasksArray(text: string): Task[] | null {
    const tasksStart = text.indexOf('"tasks"');
    if (tasksStart === -1) return null;

    const arrayStart = text.indexOf('[', tasksStart);
    if (arrayStart === -1) return null;

    let depth = 0;
    let end = -1;

    for (let i = arrayStart; i < text.length; i++) {
      const ch = text[i];
      if (ch === '[') {
        depth++;
      } else if (ch === ']') {
        depth--;
        if (depth === 0) {
          end = i + 1;
          break;
        }
      }
    }

    if (end > 0) {
      try {
        const arrayStr = text.slice(arrayStart, end);
        const tasks = JSON.parse(arrayStr) as Task[];
        if (tasks.length > 0) {
          return tasks;
        }
      } catch {
        // Failed to parse
      }
    }

    return null;
  }
}

function truncateStr(s: string, maxLen: number): string {
  if (s.length <= maxLen) return s;
  return s.slice(0, maxLen - 3) + '...';
}

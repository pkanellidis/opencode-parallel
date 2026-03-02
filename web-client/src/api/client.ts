import type {
  HealthResponse,
  Session,
  MessageResponse,
  Project,
  ProviderResponse,
  StreamEvent,
  Part,
  QuestionInfo,
} from '../types';

const BASE_URL = '/api';

async function request<T>(
  endpoint: string,
  options?: RequestInit
): Promise<T> {
  const response = await fetch(`${BASE_URL}${endpoint}`, {
    ...options,
    headers: {
      'Content-Type': 'application/json',
      ...options?.headers,
    },
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Request failed: ${response.status} - ${text}`);
  }

  return response.json();
}

export async function getHealth(): Promise<HealthResponse> {
  return request<HealthResponse>('/global/health');
}

export async function isHealthy(): Promise<boolean> {
  try {
    const health = await getHealth();
    return health.healthy;
  } catch {
    return false;
  }
}

export async function createSession(title?: string): Promise<Session> {
  return request<Session>('/session', {
    method: 'POST',
    body: JSON.stringify({ title }),
  });
}

export async function sendMessage(
  sessionId: string,
  text: string,
  model?: string
): Promise<MessageResponse> {
  const body: {
    parts: Array<{ type: string; text: string }>;
    model?: { providerID: string; modelID: string };
  } = {
    parts: [{ type: 'text', text }],
  };

  if (model) {
    const [providerId, modelId] = model.split('/');
    if (providerId && modelId) {
      body.model = { providerID: providerId, modelID: modelId };
    }
  }

  return request<MessageResponse>(`/session/${sessionId}/message`, {
    method: 'POST',
    body: JSON.stringify(body),
  });
}

export async function sendMessageAsync(
  sessionId: string,
  text: string,
  model?: string
): Promise<void> {
  const body: {
    parts: Array<{ type: string; text: string }>;
    model?: { providerID: string; modelID: string };
  } = {
    parts: [{ type: 'text', text }],
  };

  if (model) {
    const [providerId, modelId] = model.split('/');
    if (providerId && modelId) {
      body.model = { providerID: providerId, modelID: modelId };
    }
  }

  const response = await fetch(`${BASE_URL}/session/${sessionId}/prompt_async`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(body),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Async message failed: ${response.status} - ${text}`);
  }
}

export async function listProjects(): Promise<Project[]> {
  return request<Project[]>('/project');
}

export async function getCurrentProject(): Promise<Project> {
  return request<Project>('/project/current');
}

export async function getPath(): Promise<string> {
  const response = await fetch(`${BASE_URL}/path`);
  const text = await response.text();
  try {
    const parsed = JSON.parse(text);
    return parsed.path || parsed;
  } catch {
    return text.replace(/"/g, '');
  }
}

export async function getProviders(): Promise<ProviderResponse> {
  return request<ProviderResponse>('/provider');
}

export async function getConfig(): Promise<unknown> {
  return request<unknown>('/config');
}

export async function setModel(providerId: string, modelId: string): Promise<void> {
  const response = await fetch(`${BASE_URL}/config`, {
    method: 'PATCH',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model: `${providerId}/${modelId}` }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Failed to set model: ${response.status} - ${text}`);
  }
}

export async function replyToQuestion(
  requestId: string,
  answers: string[][]
): Promise<void> {
  const response = await fetch(`${BASE_URL}/question/${requestId}/reply`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ answers }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Failed to reply to question: ${response.status} - ${text}`);
  }
}

export async function replyToPermission(
  requestId: string,
  reply: 'once' | 'always' | 'reject'
): Promise<void> {
  const response = await fetch(`${BASE_URL}/permission/${requestId}/reply`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ reply }),
  });

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Failed to reply to permission: ${response.status} - ${text}`);
  }
}

function parseSSEEvent(data: string): StreamEvent | null {
  try {
    const parsed = JSON.parse(data);
    const eventType = parsed.type as string;

    switch (eventType) {
      case 'message.part.updated': {
        const props = parsed.properties;
        const partData = props?.part;
        if (!partData) return null;

        const sessionId = partData.sessionID || '';
        const partType = partData.type;

        if (partType === 'text') {
          const part: Part = {
            id: partData.id,
            type: partData.type,
            text: partData.text,
          };
          if (part.text) {
            return { type: 'part_updated', sessionId, part };
          }
        } else if (partType === 'tool') {
          const toolName = partData.tool || 'unknown';
          const status = partData.state?.status || 'unknown';
          const input = partData.state?.input || null;
          return { type: 'tool_call', sessionId, toolName, status, input };
        }
        return null;
      }

      case 'session.idle': {
        const sessionId = parsed.properties?.sessionID || '';
        return { type: 'session_idle', sessionId };
      }

      case 'question.asked': {
        const props = parsed.properties;
        const requestId = props?.id || '';
        const sessionId = props?.sessionID || '';
        const questions: QuestionInfo[] = props?.questions || [];
        return { type: 'question_asked', sessionId, requestId, questions };
      }

      case 'permission.asked': {
        const props = parsed.properties;
        const requestId = props?.id || '';
        const sessionId = props?.sessionID || '';
        const permission = props?.permission || '';
        const patterns: string[] = props?.patterns || [];
        return { type: 'permission_asked', sessionId, requestId, permission, patterns };
      }

      default:
        return null;
    }
  } catch {
    return null;
  }
}

export function subscribeToEvents(
  onEvent: (event: StreamEvent) => void
): () => void {
  const eventSource = new EventSource('/event');

  eventSource.onopen = () => {
    onEvent({ type: 'connected' });
  };

  eventSource.onmessage = (event) => {
    const parsed = parseSSEEvent(event.data);
    if (parsed) {
      onEvent(parsed);
    }
  };

  eventSource.onerror = (e) => {
    console.error('SSE error:', e);
    onEvent({ type: 'error', message: 'Connection error' });
  };

  return () => {
    eventSource.close();
  };
}

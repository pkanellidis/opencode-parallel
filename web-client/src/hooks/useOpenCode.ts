import { useState, useEffect, useCallback, useRef } from 'react';
import type {
  HealthResponse,
  Provider,
  UISession,
  Worker,
  StreamEvent,
} from '../types';
import * as api from '../api/client';
import { parseSlashCommand, type SlashCommand, COMMAND_SUGGESTIONS } from '../utils/commands';

interface UseOpenCodeState {
  connected: boolean;
  health: HealthResponse | null;
  providers: Provider[];
  connectedProviderIds: string[];
  currentModel: string | null;
  sessions: UISession[];
  activeSessionId: number | null;
  loading: boolean;
  error: string | null;
  showModelSelector: boolean;
  showStopSelector: boolean;
}

export interface CommandResult {
  success: boolean;
  message?: string;
}

export function useOpenCode() {
  const [state, setState] = useState<UseOpenCodeState>({
    connected: false,
    health: null,
    providers: [],
    connectedProviderIds: [],
    currentModel: null,
    sessions: [],
    activeSessionId: null,
    loading: true,
    error: null,
    showModelSelector: false,
    showStopSelector: false,
  });

  const unsubscribeRef = useRef<(() => void) | null>(null);
  const sessionIdMapRef = useRef<Map<string, { uiSessionId: number; workerId: number }>>(
    new Map()
  );

  const updateWorker = useCallback(
    (
      uiSessionId: number,
      workerId: number,
      updater: (worker: Worker) => Worker
    ) => {
      setState((prev) => ({
        ...prev,
        sessions: prev.sessions.map((session) =>
          session.id === uiSessionId
            ? {
                ...session,
                workers: session.workers.map((w) =>
                  w.id === workerId ? updater(w) : w
                ),
              }
            : session
        ),
      }));
    },
    []
  );

  const handleStreamEvent = useCallback(
    (event: StreamEvent) => {
      switch (event.type) {
        case 'connected':
          setState((prev) => ({ ...prev, connected: true }));
          break;

        case 'part_updated': {
          const mapping = sessionIdMapRef.current.get(event.sessionId);
          if (mapping) {
            updateWorker(mapping.uiSessionId, mapping.workerId, (w) => ({
              ...w,
              state: 'running',
              streamingContent: event.part.text || '',
            }));
          }
          break;
        }

        case 'tool_call': {
          const mapping = sessionIdMapRef.current.get(event.sessionId);
          if (mapping) {
            updateWorker(mapping.uiSessionId, mapping.workerId, (w) => ({
              ...w,
              state: 'running',
              currentTool: event.status === 'running' ? event.toolName : undefined,
              toolHistory:
                event.status === 'completed'
                  ? [...w.toolHistory, event.toolName]
                  : w.toolHistory,
            }));
          }
          break;
        }

        case 'session_idle': {
          const mapping = sessionIdMapRef.current.get(event.sessionId);
          if (mapping) {
            updateWorker(mapping.uiSessionId, mapping.workerId, (w) => ({
              ...w,
              state: 'complete',
              currentTool: undefined,
              output: w.streamingContent
                ? [...w.output, w.streamingContent]
                : w.output,
            }));
          }
          break;
        }

        case 'question_asked': {
          const mapping = sessionIdMapRef.current.get(event.sessionId);
          if (mapping && event.questions.length > 0) {
            updateWorker(mapping.uiSessionId, mapping.workerId, (w) => ({
              ...w,
              state: 'waiting_input',
              pendingQuestion: event.questions[0],
              pendingQuestionRequestId: event.requestId,
            }));
          }
          break;
        }

        case 'permission_asked': {
          const mapping = sessionIdMapRef.current.get(event.sessionId);
          if (mapping) {
            updateWorker(mapping.uiSessionId, mapping.workerId, (w) => ({
              ...w,
              state: 'waiting_input',
              pendingPermission: {
                permission: event.permission,
                patterns: event.patterns,
              },
              pendingPermissionRequestId: event.requestId,
            }));
          }
          break;
        }

        case 'error':
          console.warn('SSE connection error:', event.message);
          break;
      }
    },
    [updateWorker]
  );

  useEffect(() => {
    const init = async () => {
      try {
        const [health, providerResp] = await Promise.all([
          api.getHealth(),
          api.getProviders(),
        ]);

        setState((prev) => ({
          ...prev,
          health,
          providers: providerResp.all,
          connectedProviderIds: providerResp.connected,
          loading: false,
        }));

        unsubscribeRef.current = api.subscribeToEvents(handleStreamEvent);
      } catch (err) {
        setState((prev) => ({
          ...prev,
          loading: false,
          error: err instanceof Error ? err.message : 'Failed to connect',
        }));
      }
    };

    init();

    return () => {
      if (unsubscribeRef.current) {
        unsubscribeRef.current();
      }
    };
  }, [handleStreamEvent]);

  const addSessionMessage = useCallback((content: string, isUser: boolean) => {
    setState((prev) => {
      const activeSession = prev.sessions.find((s) => s.id === prev.activeSessionId);
      if (!activeSession) return prev;
      return {
        ...prev,
        sessions: prev.sessions.map((s) =>
          s.id === prev.activeSessionId
            ? { ...s, messages: [...s.messages, { content, isUser }] }
            : s
        ),
      };
    });
  }, []);

  const createSession = useCallback(async (name: string): Promise<UISession> => {
    const newSession: UISession = {
      id: Date.now(),
      name,
      messages: [],
      workers: [],
    };

    setState((prev) => ({
      ...prev,
      sessions: [...prev.sessions, newSession],
      activeSessionId: newSession.id,
    }));

    return newSession;
  }, []);

  const setActiveSession = useCallback((id: number) => {
    setState((prev) => ({ ...prev, activeSessionId: id }));
  }, []);

  const renameSession = useCallback((name: string) => {
    setState((prev) => ({
      ...prev,
      sessions: prev.sessions.map((s) =>
        s.id === prev.activeSessionId ? { ...s, name } : s
      ),
    }));
  }, []);

  const deleteSession = useCallback(() => {
    setState((prev) => {
      const filtered = prev.sessions.filter((s) => s.id !== prev.activeSessionId);
      return {
        ...prev,
        sessions: filtered,
        activeSessionId: filtered.length > 0 ? filtered[0].id : null,
      };
    });
  }, []);

  const clearSessionMessages = useCallback(() => {
    setState((prev) => ({
      ...prev,
      sessions: prev.sessions.map((s) =>
        s.id === prev.activeSessionId ? { ...s, messages: [] } : s
      ),
    }));
  }, []);

  const setShowModelSelector = useCallback((show: boolean) => {
    setState((prev) => ({ ...prev, showModelSelector: show }));
  }, []);

  const setShowStopSelector = useCallback((show: boolean) => {
    setState((prev) => ({ ...prev, showStopSelector: show }));
  }, []);

  const handleSlashCommand = useCallback(
    async (cmd: SlashCommand): Promise<CommandResult> => {
      switch (cmd.type) {
        case 'help': {
          const helpText = COMMAND_SUGGESTIONS.map(
            (s) => `${s.command} - ${s.description}`
          ).join('\n');
          addSessionMessage(`Available commands:\n${helpText}`, false);
          return { success: true };
        }

        case 'new_session': {
          const name = cmd.name || `Session ${state.sessions.length + 1}`;
          await createSession(name);
          addSessionMessage(`Created new session: ${name}`, false);
          return { success: true };
        }

        case 'list_sessions': {
          const list = state.sessions
            .map((s, i) => `${i + 1}. ${s.name} (${s.workers.length} workers)`)
            .join('\n');
          addSessionMessage(
            state.sessions.length > 0 ? `Sessions:\n${list}` : 'No sessions.',
            false
          );
          return { success: true };
        }

        case 'rename_session': {
          if (!state.activeSessionId) {
            return { success: false, message: 'No active session to rename' };
          }
          renameSession(cmd.name);
          addSessionMessage(`Session renamed to: ${cmd.name}`, false);
          return { success: true };
        }

        case 'delete_session': {
          if (!state.activeSessionId) {
            return { success: false, message: 'No active session to delete' };
          }
          deleteSession();
          addSessionMessage('Session deleted.', false);
          return { success: true };
        }

        case 'clear': {
          clearSessionMessages();
          return { success: true };
        }

        case 'models': {
          try {
            const providerResp = await api.getProviders();
            const lines: string[] = [];
            for (const provider of providerResp.all) {
              const connected = providerResp.connected.includes(provider.id);
              const status = connected ? '✓' : '✗';
              lines.push(`${status} ${provider.name || provider.id}`);
              for (const [modelId, model] of Object.entries(provider.models)) {
                lines.push(`    - ${model.name || modelId}`);
              }
            }
            addSessionMessage(`Models:\n${lines.join('\n')}`, false);
            return { success: true };
          } catch (err) {
            return {
              success: false,
              message: err instanceof Error ? err.message : 'Failed to fetch models',
            };
          }
        }

        case 'model_select': {
          setShowModelSelector(true);
          return { success: true };
        }

        case 'model_set': {
          try {
            await api.setModel(cmd.providerId, cmd.modelId);
            setState((prev) => ({
              ...prev,
              currentModel: `${cmd.providerId}/${cmd.modelId}`,
            }));
            addSessionMessage(`Model set to: ${cmd.providerId}/${cmd.modelId}`, false);
            return { success: true };
          } catch (err) {
            return {
              success: false,
              message: err instanceof Error ? err.message : 'Failed to set model',
            };
          }
        }

        case 'projects': {
          try {
            const projects = await api.listProjects();
            const list = projects
              .map((p) => `- ${p.worktree}${p.vcs ? ` (${p.vcs})` : ''}`)
              .join('\n');
            addSessionMessage(
              projects.length > 0 ? `Projects:\n${list}` : 'No projects found.',
              false
            );
            return { success: true };
          } catch (err) {
            return {
              success: false,
              message: err instanceof Error ? err.message : 'Failed to fetch projects',
            };
          }
        }

        case 'project_current': {
          try {
            const project = await api.getCurrentProject();
            addSessionMessage(
              `Current project: ${project.worktree}${project.vcs ? ` (${project.vcs})` : ''}`,
              false
            );
            return { success: true };
          } catch (err) {
            return {
              success: false,
              message: err instanceof Error ? err.message : 'Failed to get current project',
            };
          }
        }

        case 'path': {
          try {
            const path = await api.getPath();
            addSessionMessage(`Current path: ${path}`, false);
            return { success: true };
          } catch (err) {
            return {
              success: false,
              message: err instanceof Error ? err.message : 'Failed to get path',
            };
          }
        }

        case 'config': {
          try {
            const config = await api.getConfig();
            addSessionMessage(`Config:\n${JSON.stringify(config, null, 2)}`, false);
            return { success: true };
          } catch (err) {
            return {
              success: false,
              message: err instanceof Error ? err.message : 'Failed to get config',
            };
          }
        }

        case 'stop': {
          setShowStopSelector(true);
          return { success: true };
        }

        case 'reply': {
          const activeSession = state.sessions.find((s) => s.id === state.activeSessionId);
          if (!activeSession) {
            return { success: false, message: 'No active session' };
          }
          const workerIndex = cmd.workerId - 1;
          const worker = activeSession.workers[workerIndex];
          if (!worker) {
            return { success: false, message: `Worker #${cmd.workerId} not found` };
          }
          if (worker.pendingQuestionRequestId) {
            const answer = cmd.message || '';
            await api.replyToQuestion(worker.pendingQuestionRequestId, [[answer]]);
            updateWorker(activeSession.id, worker.id, (w) => ({
              ...w,
              state: 'running',
              pendingQuestion: undefined,
              pendingQuestionRequestId: undefined,
            }));
            addSessionMessage(`Replied to worker #${cmd.workerId}: ${answer}`, false);
            return { success: true };
          } else if (worker.sessionId && cmd.message) {
            await api.sendMessageAsync(worker.sessionId, cmd.message, state.currentModel || undefined);
            updateWorker(activeSession.id, worker.id, (w) => ({ ...w, state: 'running' }));
            addSessionMessage(`Sent to worker #${cmd.workerId}: ${cmd.message}`, false);
            return { success: true };
          }
          return { success: false, message: `Worker #${cmd.workerId} has no pending question` };
        }

        case 'unknown': {
          addSessionMessage(`Unknown command: ${cmd.command}`, false);
          return { success: false, message: `Unknown command: ${cmd.command}` };
        }

        default:
          return { success: false, message: 'Unhandled command' };
      }
    },
    [
      state.sessions,
      state.activeSessionId,
      state.currentModel,
      addSessionMessage,
      createSession,
      renameSession,
      deleteSession,
      clearSessionMessages,
      setShowModelSelector,
      setShowStopSelector,
      updateWorker,
    ]
  );

  const handleInput = useCallback(
    async (input: string) => {
      const cmd = parseSlashCommand(input);
      if (cmd) {
        addSessionMessage(`> ${input}`, true);
        const result = await handleSlashCommand(cmd);
        if (!result.success && result.message) {
          addSessionMessage(`Error: ${result.message}`, false);
        }
        return;
      }

      if (!state.activeSessionId) {
        return;
      }

      const session = await api.createSession(`Task: ${input.slice(0, 50)}`);
      const workerId = Date.now();

      const worker: Worker = {
        id: workerId,
        description: input,
        sessionId: session.id,
        state: 'starting',
        output: [],
        streamingContent: '',
        toolHistory: [],
      };

      sessionIdMapRef.current.set(session.id, { uiSessionId: state.activeSessionId, workerId });

      setState((prev) => ({
        ...prev,
        sessions: prev.sessions.map((s) =>
          s.id === state.activeSessionId ? { ...s, workers: [...s.workers, worker] } : s
        ),
      }));

      await api.sendMessageAsync(session.id, input, state.currentModel || undefined);

      updateWorker(state.activeSessionId, workerId, (w) => ({ ...w, state: 'running' }));
    },
    [state.activeSessionId, state.currentModel, addSessionMessage, handleSlashCommand, updateWorker]
  );

  const answerQuestion = useCallback(
    async (
      uiSessionId: number,
      workerId: number,
      requestId: string,
      answers: string[]
    ) => {
      await api.replyToQuestion(requestId, [answers]);
      updateWorker(uiSessionId, workerId, (w) => ({
        ...w,
        state: 'running',
        pendingQuestion: undefined,
        pendingQuestionRequestId: undefined,
      }));
    },
    [updateWorker]
  );

  const answerPermission = useCallback(
    async (
      uiSessionId: number,
      workerId: number,
      requestId: string,
      reply: 'once' | 'always' | 'reject'
    ) => {
      await api.replyToPermission(requestId, reply);
      updateWorker(uiSessionId, workerId, (w) => ({
        ...w,
        state: 'running',
        pendingPermission: undefined,
        pendingPermissionRequestId: undefined,
      }));
    },
    [updateWorker]
  );

  const setModel = useCallback(async (providerId: string, modelId: string) => {
    await api.setModel(providerId, modelId);
    setState((prev) => ({
      ...prev,
      currentModel: `${providerId}/${modelId}`,
      showModelSelector: false,
    }));
  }, []);

  const activeSession = state.sessions.find((s) => s.id === state.activeSessionId);

  return {
    ...state,
    activeSession,
    createSession,
    setActiveSession,
    handleInput,
    answerQuestion,
    answerPermission,
    setModel,
    setShowModelSelector,
    setShowStopSelector,
  };
}

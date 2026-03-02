export type SlashCommand =
  | { type: 'help' }
  | { type: 'projects' }
  | { type: 'project_current' }
  | { type: 'path' }
  | { type: 'clear' }
  | { type: 'new_session'; name?: string }
  | { type: 'list_sessions' }
  | { type: 'rename_session'; name: string }
  | { type: 'delete_session' }
  | { type: 'models' }
  | { type: 'model_select' }
  | { type: 'model_set'; providerId: string; modelId: string }
  | { type: 'reply'; workerId: number; message?: string }
  | { type: 'config' }
  | { type: 'stop' }
  | { type: 'unknown'; command: string };

export interface CommandSuggestion {
  command: string;
  description: string;
}

export const COMMAND_SUGGESTIONS: CommandSuggestion[] = [
  { command: '/help', description: 'Show available commands' },
  { command: '/new', description: 'Create new session' },
  { command: '/sessions', description: 'List all sessions' },
  { command: '/rename', description: 'Rename current session' },
  { command: '/delete', description: 'Delete current session' },
  { command: '/models', description: 'List available models' },
  { command: '/model', description: 'Set model (provider/model)' },
  { command: '/reply', description: 'Reply to worker (/reply #N [message])' },
  { command: '/projects', description: 'List all projects' },
  { command: '/project current', description: 'Show current project' },
  { command: '/path', description: 'Show current working path' },
  { command: '/clear', description: 'Clear chat messages' },
  { command: '/config', description: 'Show current server config' },
  { command: '/stop', description: 'Stop running workers' },
];

export function getSuggestions(input: string): CommandSuggestion[] {
  if (!input.startsWith('/')) {
    return [];
  }
  const search = input.toLowerCase();
  return COMMAND_SUGGESTIONS.filter((s) => s.command.startsWith(search));
}

export function parseSlashCommand(input: string): SlashCommand | null {
  const trimmed = input.trim();
  if (!trimmed.startsWith('/')) {
    return null;
  }

  const parts = trimmed.slice(1).split(/\s+/);
  if (parts.length === 0 || (parts.length === 1 && parts[0] === '')) {
    return { type: 'help' };
  }

  const cmd = parts[0].toLowerCase();

  switch (cmd) {
    case 'help':
    case 'h':
    case '?':
      return { type: 'help' };

    case 'new':
    case 'n':
      return {
        type: 'new_session',
        name: parts.length > 1 ? parts.slice(1).join(' ') : undefined,
      };

    case 'sessions':
    case 'ls':
      return { type: 'list_sessions' };

    case 'rename':
    case 'mv':
      if (parts.length > 1) {
        return { type: 'rename_session', name: parts.slice(1).join(' ') };
      }
      return { type: 'unknown', command: 'rename requires a name' };

    case 'delete':
    case 'del':
    case 'rm':
      return { type: 'delete_session' };

    case 'models':
      return { type: 'models' };

    case 'model':
    case 'm':
      if (parts.length > 1) {
        const modelSpec = parts.slice(1).join(' ');
        const slashIdx = modelSpec.indexOf('/');
        if (slashIdx > 0) {
          return {
            type: 'model_set',
            providerId: modelSpec.slice(0, slashIdx),
            modelId: modelSpec.slice(slashIdx + 1),
          };
        }
        return { type: 'unknown', command: 'model requires provider/model format' };
      }
      return { type: 'model_select' };

    case 'projects':
    case 'project':
    case 'proj':
    case 'p':
      if (parts.length > 1 && parts[1] === 'current') {
        return { type: 'project_current' };
      }
      return { type: 'projects' };

    case 'path':
    case 'pwd':
      return { type: 'path' };

    case 'clear':
    case 'cls':
      return { type: 'clear' };

    case 'config':
    case 'cfg':
      return { type: 'config' };

    case 'stop':
    case 's':
    case 'kill':
    case 'cancel':
      return { type: 'stop' };

    case 'reply':
    case 'r':
      if (parts.length >= 2) {
        const workerStr = parts[1].replace(/^#/, '');
        const workerId = parseInt(workerStr, 10);
        if (!isNaN(workerId)) {
          return {
            type: 'reply',
            workerId,
            message: parts.length > 2 ? parts.slice(2).join(' ') : undefined,
          };
        }
        return {
          type: 'unknown',
          command: 'reply requires worker number (e.g., /reply #1 or /reply #1 message)',
        };
      }
      return {
        type: 'unknown',
        command: 'reply requires worker number (e.g., /reply #1 or /reply #1 message)',
      };

    default:
      return { type: 'unknown', command: cmd };
  }
}

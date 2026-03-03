import { useState } from 'react';
import type {
  ToolCallDetails,
  EditToolInput,
  WriteToolInput,
  ReadToolInput,
  BashToolInput,
  GlobToolInput,
  GrepToolInput,
} from '../types';
import { DiffViewer } from './DiffViewer';
import { CodeBlock } from './CodeBlock';

interface ToolCallDisplayProps {
  toolCall: ToolCallDetails;
  defaultExpanded?: boolean;
}

const TOOL_ICONS: Record<string, string> = {
  edit: '✏️',
  write: '📝',
  read: '📖',
  bash: '💻',
  glob: '🔍',
  grep: '🔎',
  task: '📋',
  todowrite: '✅',
  webfetch: '🌐',
  question: '❓',
  default: '🔧',
};

const STATUS_COLORS: Record<string, string> = {
  running: '#f59e0b',
  completed: '#10b981',
  error: '#ef4444',
};

function getToolIcon(toolName: string): string {
  const name = toolName.toLowerCase();
  return TOOL_ICONS[name] || TOOL_ICONS.default;
}

function formatDuration(startTime: number, endTime?: number): string {
  if (!endTime) return 'running...';
  const duration = endTime - startTime;
  if (duration < 1000) return `${duration}ms`;
  return `${(duration / 1000).toFixed(2)}s`;
}

function isEditToolInput(input: unknown): input is EditToolInput {
  return (
    typeof input === 'object' &&
    input !== null &&
    'filePath' in input &&
    'oldString' in input &&
    'newString' in input
  );
}

function isWriteToolInput(input: unknown): input is WriteToolInput {
  return (
    typeof input === 'object' &&
    input !== null &&
    'filePath' in input &&
    'content' in input &&
    !('oldString' in input)
  );
}

function isReadToolInput(input: unknown): input is ReadToolInput {
  return (
    typeof input === 'object' &&
    input !== null &&
    'filePath' in input &&
    !('content' in input) &&
    !('oldString' in input)
  );
}

function isBashToolInput(input: unknown): input is BashToolInput {
  return typeof input === 'object' && input !== null && 'command' in input;
}

function isGlobToolInput(input: unknown): input is GlobToolInput {
  return typeof input === 'object' && input !== null && 'pattern' in input && !('include' in input);
}

function isGrepToolInput(input: unknown): input is GrepToolInput {
  return typeof input === 'object' && input !== null && 'pattern' in input;
}

function EditToolDisplay({ input }: { input: EditToolInput }) {
  return (
    <div className="tool-call-detail edit-detail">
      <div className="tool-call-file-path">
        <span className="label">File:</span>
        <span className="value">{input.filePath}</span>
      </div>
      <DiffViewer
        oldValue={input.oldString}
        newValue={input.newString}
        filename={input.filePath}
      />
    </div>
  );
}

function WriteToolDisplay({ input }: { input: WriteToolInput }) {
  return (
    <div className="tool-call-detail write-detail">
      <div className="tool-call-file-path">
        <span className="label">File:</span>
        <span className="value">{input.filePath}</span>
      </div>
      <div className="tool-call-content">
        <CodeBlock code={input.content} filename={input.filePath} />
      </div>
    </div>
  );
}

function ReadToolDisplay({ input, output }: { input: ReadToolInput; output?: string }) {
  return (
    <div className="tool-call-detail read-detail">
      <div className="tool-call-file-path">
        <span className="label">File:</span>
        <span className="value">{input.filePath}</span>
        {input.offset && <span className="meta">offset: {input.offset}</span>}
        {input.limit && <span className="meta">limit: {input.limit}</span>}
      </div>
      {output && (
        <div className="tool-call-output">
          <CodeBlock
            code={output.slice(0, 2000) + (output.length > 2000 ? '\n... (truncated)' : '')}
            filename={input.filePath}
          />
        </div>
      )}
    </div>
  );
}

function BashToolDisplay({ input, output }: { input: BashToolInput; output?: string }) {
  return (
    <div className="tool-call-detail bash-detail">
      <div className="tool-call-command">
        <span className="label">Command:</span>
        <code className="value">{input.command}</code>
      </div>
      {input.workdir && (
        <div className="tool-call-workdir">
          <span className="label">Working Dir:</span>
          <span className="value">{input.workdir}</span>
        </div>
      )}
      {input.description && (
        <div className="tool-call-description">
          <span className="label">Description:</span>
          <span className="value">{input.description}</span>
        </div>
      )}
      {output && (
        <div className="tool-call-output">
          <div className="output-label">Output:</div>
          <CodeBlock
            code={output.slice(0, 2000) + (output.length > 2000 ? '\n... (truncated)' : '')}
            language="bash"
          />
        </div>
      )}
    </div>
  );
}

function GlobToolDisplay({ input, output }: { input: GlobToolInput; output?: string }) {
  return (
    <div className="tool-call-detail glob-detail">
      <div className="tool-call-pattern">
        <span className="label">Pattern:</span>
        <code className="value">{input.pattern}</code>
      </div>
      {input.path && (
        <div className="tool-call-path">
          <span className="label">Path:</span>
          <span className="value">{input.path}</span>
        </div>
      )}
      {output && (
        <div className="tool-call-output">
          <div className="output-label">Matches:</div>
          <pre className="output-content">{output}</pre>
        </div>
      )}
    </div>
  );
}

function GrepToolDisplay({ input, output }: { input: GrepToolInput; output?: string }) {
  return (
    <div className="tool-call-detail grep-detail">
      <div className="tool-call-pattern">
        <span className="label">Pattern:</span>
        <code className="value">{input.pattern}</code>
      </div>
      {input.path && (
        <div className="tool-call-path">
          <span className="label">Path:</span>
          <span className="value">{input.path}</span>
        </div>
      )}
      {input.include && (
        <div className="tool-call-include">
          <span className="label">Include:</span>
          <code className="value">{input.include}</code>
        </div>
      )}
      {output && (
        <div className="tool-call-output">
          <div className="output-label">Results:</div>
          <pre className="output-content">{output}</pre>
        </div>
      )}
    </div>
  );
}

function GenericToolDisplay({ input, output }: { input: unknown; output?: string }) {
  return (
    <div className="tool-call-detail generic-detail">
      <div className="tool-call-input">
        <div className="output-label">Input:</div>
        <pre className="output-content">
          {JSON.stringify(input, null, 2)}
        </pre>
      </div>
      {output && (
        <div className="tool-call-output">
          <div className="output-label">Output:</div>
          <pre className="output-content">
            {output.slice(0, 1000) + (output.length > 1000 ? '\n... (truncated)' : '')}
          </pre>
        </div>
      )}
    </div>
  );
}

export function ToolCallDisplay({ toolCall, defaultExpanded = false }: ToolCallDisplayProps) {
  const [isExpanded, setIsExpanded] = useState(defaultExpanded);

  const renderToolDetails = () => {
    const { input, output, toolName } = toolCall;
    const name = toolName.toLowerCase();

    if (name === 'edit' && isEditToolInput(input)) {
      return <EditToolDisplay input={input} />;
    }

    if (name === 'write' && isWriteToolInput(input)) {
      return <WriteToolDisplay input={input} />;
    }

    if (name === 'read' && isReadToolInput(input)) {
      return <ReadToolDisplay input={input} output={output} />;
    }

    if (name === 'bash' && isBashToolInput(input)) {
      return <BashToolDisplay input={input} output={output} />;
    }

    if (name === 'glob' && isGlobToolInput(input)) {
      return <GlobToolDisplay input={input} output={output} />;
    }

    if (name === 'grep' && isGrepToolInput(input)) {
      return <GrepToolDisplay input={input} output={output} />;
    }

    return <GenericToolDisplay input={input} output={output} />;
  };

  const getToolSummary = (): string => {
    const { input, toolName } = toolCall;
    const name = toolName.toLowerCase();

    if ((name === 'edit' || name === 'write' || name === 'read') && typeof input === 'object' && input !== null && 'filePath' in input) {
      return (input as { filePath: string }).filePath;
    }

    if (name === 'bash' && typeof input === 'object' && input !== null && 'command' in input) {
      const cmd = (input as BashToolInput).command;
      return cmd.length > 50 ? cmd.slice(0, 50) + '...' : cmd;
    }

    if ((name === 'glob' || name === 'grep') && typeof input === 'object' && input !== null && 'pattern' in input) {
      return (input as { pattern: string }).pattern;
    }

    return '';
  };

  return (
    <div className={`tool-call-item ${toolCall.status}`}>
      <button
        className="tool-call-header"
        onClick={() => setIsExpanded(!isExpanded)}
        type="button"
      >
        <div className="tool-call-left">
          <span className="tool-call-icon">{getToolIcon(toolCall.toolName)}</span>
          <span className="tool-call-name">{toolCall.toolName}</span>
          <span className="tool-call-summary">{getToolSummary()}</span>
        </div>
        <div className="tool-call-right">
          <span
            className="tool-call-status"
            style={{ color: STATUS_COLORS[toolCall.status] }}
          >
            {toolCall.status}
          </span>
          <span className="tool-call-duration">
            {formatDuration(toolCall.startTime, toolCall.endTime)}
          </span>
          <span className={`tool-call-expand ${isExpanded ? 'expanded' : ''}`}>
            ▶
          </span>
        </div>
      </button>

      {isExpanded && (
        <div className="tool-call-body">
          {toolCall.error ? (
            <div className="tool-call-error">
              <span className="error-label">Error:</span>
              <pre className="error-content">{toolCall.error}</pre>
            </div>
          ) : (
            renderToolDetails()
          )}
        </div>
      )}
    </div>
  );
}

interface ToolCallListProps {
  toolCalls: ToolCallDetails[];
  maxVisible?: number;
}

export function ToolCallList({ toolCalls, maxVisible = 10 }: ToolCallListProps) {
  const [showAll, setShowAll] = useState(false);
  const displayedCalls = showAll ? toolCalls : toolCalls.slice(-maxVisible);
  const hiddenCount = toolCalls.length - maxVisible;

  if (toolCalls.length === 0) {
    return null;
  }

  return (
    <div className="tool-call-list">
      <div className="tool-call-list-header">
        <span className="tool-call-count">{toolCalls.length} tool calls</span>
        {hiddenCount > 0 && !showAll && (
          <button
            className="show-all-btn"
            onClick={() => setShowAll(true)}
            type="button"
          >
            Show {hiddenCount} more
          </button>
        )}
        {showAll && hiddenCount > 0 && (
          <button
            className="show-all-btn"
            onClick={() => setShowAll(false)}
            type="button"
          >
            Show less
          </button>
        )}
      </div>
      <div className="tool-call-list-items">
        {displayedCalls.map((tc) => (
          <ToolCallDisplay
            key={tc.id}
            toolCall={tc}
            defaultExpanded={tc.status === 'running'}
          />
        ))}
      </div>
    </div>
  );
}

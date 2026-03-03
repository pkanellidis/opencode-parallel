import ReactMarkdown from 'react-markdown';
import type { Worker } from '../types';
import { ToolCallList } from './ToolCallDisplay';
import { CodeBlock } from './CodeBlock';

interface WorkerCardProps {
  worker: Worker;
  onAnswerQuestion: (requestId: string, answers: string[]) => void;
  onAnswerPermission: (requestId: string, reply: 'once' | 'always' | 'reject') => void;
}

const STATE_LABELS: Record<string, string> = {
  idle: 'Idle',
  starting: 'Starting...',
  running: 'Running',
  waiting_input: 'Waiting for input',
  complete: 'Complete',
  error: 'Error',
};

const STATE_COLORS: Record<string, string> = {
  idle: '#6b7280',
  starting: '#f59e0b',
  running: '#3b82f6',
  waiting_input: '#8b5cf6',
  complete: '#10b981',
  error: '#ef4444',
};

export function WorkerCard({
  worker,
  onAnswerQuestion,
  onAnswerPermission,
}: WorkerCardProps) {
  const content = worker.streamingContent || worker.output.join('\n');

  return (
    <div className="worker-card">
      <div className="worker-header">
        <div className="worker-info">
          <span className="worker-id">Worker #{worker.id}</span>
          <span
            className="worker-state"
            style={{ color: STATE_COLORS[worker.state] }}
          >
            {STATE_LABELS[worker.state]}
          </span>
        </div>
        {worker.currentTool && (
          <span className="worker-tool">
            Running: {worker.currentTool}
          </span>
        )}
      </div>

      <div className="worker-description">{worker.description}</div>

      {worker.toolCalls.length > 0 && (
        <ToolCallList toolCalls={worker.toolCalls} maxVisible={5} />
      )}

      {worker.toolCalls.length === 0 && worker.toolHistory.length > 0 && (
        <div className="worker-tools">
          <span className="tools-label">Tools used:</span>
          <div className="tools-list">
            {worker.toolHistory.slice(-5).map((tool, i) => (
              <span key={i} className="tool-badge">
                {tool}
              </span>
            ))}
            {worker.toolHistory.length > 5 && (
              <span className="tool-badge more">
                +{worker.toolHistory.length - 5} more
              </span>
            )}
          </div>
        </div>
      )}

      {content && (
        <div className="worker-content">
          <ReactMarkdown
            components={{
              code({ className, children, ...props }) {
                const match = /language-(\w+)/.exec(className || '');
                const codeString = String(children).replace(/\n$/, '');
                if (match && codeString.includes('\n')) {
                  return (
                    <CodeBlock
                      code={codeString}
                      language={match[1]}
                    />
                  );
                }
                return (
                  <code className={className} {...props}>
                    {children}
                  </code>
                );
              },
            }}
          >
            {content}
          </ReactMarkdown>
        </div>
      )}

      {worker.pendingQuestion && worker.pendingQuestionRequestId && (
        <div className="worker-question">
          <p className="question-text">{worker.pendingQuestion.question}</p>
          <div className="question-options">
            {worker.pendingQuestion.options.map((opt) => (
              <button
                key={opt.label}
                className="btn-option"
                onClick={() =>
                  onAnswerQuestion(worker.pendingQuestionRequestId!, [opt.label])
                }
                title={opt.description}
              >
                {opt.label}
              </button>
            ))}
          </div>
        </div>
      )}

      {worker.pendingPermission && worker.pendingPermissionRequestId && (
        <div className="worker-permission">
          <p className="permission-text">
            Permission requested: <strong>{worker.pendingPermission.permission}</strong>
          </p>
          {worker.pendingPermission.patterns.length > 0 && (
            <ul className="permission-patterns">
              {worker.pendingPermission.patterns.map((p, i) => (
                <li key={i}>{p}</li>
              ))}
            </ul>
          )}
          <div className="permission-actions">
            <button
              className="btn-permission allow-once"
              onClick={() =>
                onAnswerPermission(worker.pendingPermissionRequestId!, 'once')
              }
            >
              Allow Once
            </button>
            <button
              className="btn-permission allow-always"
              onClick={() =>
                onAnswerPermission(worker.pendingPermissionRequestId!, 'always')
              }
            >
              Always Allow
            </button>
            <button
              className="btn-permission reject"
              onClick={() =>
                onAnswerPermission(worker.pendingPermissionRequestId!, 'reject')
              }
            >
              Reject
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

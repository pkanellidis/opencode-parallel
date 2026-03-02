import type { UISession } from '../types';
import { TaskInput } from './TaskInput';
import { WorkerGrid } from './WorkerGrid';

interface MainContentProps {
  session: UISession | undefined;
  onSendTask: (task: string) => void;
  onAnswerQuestion: (
    workerId: number,
    requestId: string,
    answers: string[]
  ) => void;
  onAnswerPermission: (
    workerId: number,
    requestId: string,
    reply: 'once' | 'always' | 'reject'
  ) => void;
}

export function MainContent({
  session,
  onSendTask,
  onAnswerQuestion,
  onAnswerPermission,
}: MainContentProps) {
  if (!session) {
    return (
      <main className="main-content empty">
        <div className="empty-state">
          <h2>Welcome to OpenCode Parallel</h2>
          <p>Create a new session to start running parallel AI workers.</p>
          <p className="hint">Type / for available commands.</p>
        </div>
      </main>
    );
  }

  return (
    <main className="main-content">
      <div className="session-header">
        <h2>{session.name}</h2>
      </div>

      {session.messages.length > 0 && (
        <div className="session-messages">
          {session.messages.map((msg, idx) => (
            <div
              key={idx}
              className={`session-message ${msg.isUser ? 'user' : 'system'}`}
            >
              <pre>{msg.content}</pre>
            </div>
          ))}
        </div>
      )}

      <WorkerGrid
        workers={session.workers}
        onAnswerQuestion={onAnswerQuestion}
        onAnswerPermission={onAnswerPermission}
      />

      <TaskInput onSubmit={onSendTask} />
    </main>
  );
}

import type { UISession } from '../types';

interface SidebarProps {
  sessions: UISession[];
  activeSessionId: number | null;
  onSelectSession: (id: number) => void;
  onNewSession: () => void;
}

export function Sidebar({
  sessions,
  activeSessionId,
  onSelectSession,
  onNewSession,
}: SidebarProps) {
  return (
    <aside className="sidebar">
      <div className="sidebar-header">
        <h2>Sessions</h2>
        <button className="btn-new-session" onClick={onNewSession}>
          + New
        </button>
      </div>
      <nav className="session-list">
        {sessions.length === 0 ? (
          <p className="no-sessions">No sessions yet</p>
        ) : (
          sessions.map((session) => (
            <button
              key={session.id}
              className={`session-item ${session.id === activeSessionId ? 'active' : ''}`}
              onClick={() => onSelectSession(session.id)}
            >
              <span className="session-name">{session.name}</span>
              <span className="session-workers">
                {session.workers.length} worker{session.workers.length !== 1 ? 's' : ''}
              </span>
            </button>
          ))
        )}
      </nav>
    </aside>
  );
}

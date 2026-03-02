import { useCallback } from 'react';
import { useOpenCode } from './hooks/useOpenCode';
import { Header } from './components/Header';
import { Sidebar } from './components/Sidebar';
import { MainContent } from './components/MainContent';
import { ModelSelector } from './components/ModelSelector';
import './App.css';

export default function App() {
  const {
    connected,
    health,
    providers,
    connectedProviderIds,
    currentModel,
    sessions,
    activeSessionId,
    activeSession,
    loading,
    error,
    showModelSelector,
    createSession,
    setActiveSession,
    handleInput,
    answerQuestion,
    answerPermission,
    setModel,
    setShowModelSelector,
  } = useOpenCode();

  const handleNewSession = useCallback(async () => {
    const name = `Session ${sessions.length + 1}`;
    await createSession(name);
  }, [sessions.length, createSession]);

  const handleAnswerQuestion = useCallback(
    async (workerId: number, requestId: string, answers: string[]) => {
      if (activeSessionId) {
        await answerQuestion(activeSessionId, workerId, requestId, answers);
      }
    },
    [activeSessionId, answerQuestion]
  );

  const handleAnswerPermission = useCallback(
    async (
      workerId: number,
      requestId: string,
      reply: 'once' | 'always' | 'reject'
    ) => {
      if (activeSessionId) {
        await answerPermission(activeSessionId, workerId, requestId, reply);
      }
    },
    [activeSessionId, answerPermission]
  );

  if (loading) {
    return (
      <div className="app loading">
        <div className="loader">
          <div className="spinner" />
          <p>Connecting to OpenCode server...</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="app error">
        <div className="error-state">
          <h2>Connection Error</h2>
          <p>{error}</p>
          <p className="error-hint">
            Make sure the OpenCode server is running on port 14096.
          </p>
          <button onClick={() => window.location.reload()}>Retry</button>
        </div>
      </div>
    );
  }

  return (
    <div className="app">
      <Header
        health={health}
        connected={connected}
        providers={providers}
        connectedProviderIds={connectedProviderIds}
        currentModel={currentModel}
        onModelChange={setModel}
      />
      <div className="app-body">
        <Sidebar
          sessions={sessions}
          activeSessionId={activeSessionId}
          onSelectSession={setActiveSession}
          onNewSession={handleNewSession}
        />
        <MainContent
          session={activeSession}
          onSendTask={handleInput}
          onAnswerQuestion={handleAnswerQuestion}
          onAnswerPermission={handleAnswerPermission}
        />
      </div>

      {showModelSelector && (
        <ModelSelector
          providers={providers}
          connectedProviderIds={connectedProviderIds}
          currentModel={currentModel}
          onSelect={setModel}
          onClose={() => setShowModelSelector(false)}
        />
      )}
    </div>
  );
}

import type { HealthResponse, Provider } from '../types';

interface HeaderProps {
  health: HealthResponse | null;
  connected: boolean;
  providers: Provider[];
  connectedProviderIds: string[];
  currentModel: string | null;
  onModelChange: (providerId: string, modelId: string) => void;
}

export function Header({
  health,
  connected,
  providers,
  connectedProviderIds,
  currentModel,
  onModelChange,
}: HeaderProps) {
  const connectedProviders = providers.filter(
    (p) => connectedProviderIds.includes(p.id) && Object.keys(p.models).length > 0
  );

  return (
    <header className="header">
      <div className="header-left">
        <h1 className="logo">OpenCode Parallel</h1>
        <div className="status">
          <span className={`status-dot ${connected ? 'connected' : 'disconnected'}`} />
          <span className="status-text">
            {connected ? `v${health?.version || '?'}` : 'Disconnected'}
          </span>
        </div>
      </div>

      <div className="header-right">
        <label className="model-selector">
          <span className="model-label">Model:</span>
          <select
            value={currentModel || ''}
            onChange={(e) => {
              const [provider, model] = e.target.value.split('/');
              if (provider && model) {
                onModelChange(provider, model);
              }
            }}
          >
            <option value="">Select model...</option>
            {connectedProviders.map((provider) => (
              <optgroup key={provider.id} label={provider.name || provider.id}>
                {Object.entries(provider.models).map(([id, model]) => (
                  <option key={id} value={`${provider.id}/${id}`}>
                    {model.name || id}
                  </option>
                ))}
              </optgroup>
            ))}
          </select>
        </label>
      </div>
    </header>
  );
}

import { useCallback } from 'react';
import type { Provider } from '../types';

interface ModelSelectorProps {
  providers: Provider[];
  connectedProviderIds: string[];
  currentModel: string | null;
  onSelect: (providerId: string, modelId: string) => void;
  onClose: () => void;
}

export function ModelSelector({
  providers,
  connectedProviderIds,
  currentModel,
  onSelect,
  onClose,
}: ModelSelectorProps) {
  const connectedProviders = providers.filter(
    (p) => connectedProviderIds.includes(p.id) && Object.keys(p.models).length > 0
  );

  const handleSelect = useCallback(
    (providerId: string, modelId: string) => {
      onSelect(providerId, modelId);
      onClose();
    },
    [onSelect, onClose]
  );

  return (
    <div className="modal-overlay" onClick={onClose}>
      <div className="modal model-selector-modal" onClick={(e) => e.stopPropagation()}>
        <div className="modal-header">
          <h2>Select Model</h2>
          <button className="modal-close" onClick={onClose}>
            &times;
          </button>
        </div>
        <div className="modal-body">
          {connectedProviders.length === 0 ? (
            <p className="no-providers">No providers connected.</p>
          ) : (
            connectedProviders.map((provider) => (
              <div key={provider.id} className="provider-group">
                <h3 className="provider-name">{provider.name || provider.id}</h3>
                <div className="model-list">
                  {Object.entries(provider.models).map(([modelId, model]) => {
                    const fullId = `${provider.id}/${modelId}`;
                    const isActive = currentModel === fullId;
                    return (
                      <button
                        key={modelId}
                        className={`model-item ${isActive ? 'active' : ''}`}
                        onClick={() => handleSelect(provider.id, modelId)}
                      >
                        {model.name || modelId}
                        {isActive && <span className="check-mark">✓</span>}
                      </button>
                    );
                  })}
                </div>
              </div>
            ))
          )}
        </div>
      </div>
    </div>
  );
}

import { useState, useCallback, useRef, useEffect } from 'react';
import { getSuggestions, type CommandSuggestion } from '../utils/commands';

interface TaskInputProps {
  onSubmit: (task: string) => void;
  disabled?: boolean;
}

export function TaskInput({ onSubmit, disabled }: TaskInputProps) {
  const [value, setValue] = useState('');
  const [suggestions, setSuggestions] = useState<CommandSuggestion[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [showSuggestions, setShowSuggestions] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const suggestionRefs = useRef<(HTMLButtonElement | null)[]>([]);

  useEffect(() => {
    if (value.startsWith('/')) {
      const matches = getSuggestions(value);
      setSuggestions(matches);
      setShowSuggestions(matches.length > 0);
      setSelectedIndex(0);
    } else {
      setSuggestions([]);
      setShowSuggestions(false);
    }
  }, [value]);

  useEffect(() => {
    suggestionRefs.current[selectedIndex]?.scrollIntoView({
      block: 'nearest',
      behavior: 'smooth',
    });
  }, [selectedIndex]);

  const handleSubmit = useCallback(
    (e: React.FormEvent) => {
      e.preventDefault();
      const trimmed = value.trim();
      if (trimmed && !disabled) {
        onSubmit(trimmed);
        setValue('');
        setShowSuggestions(false);
      }
    },
    [value, disabled, onSubmit]
  );

  const applySuggestion = useCallback((suggestion: CommandSuggestion) => {
    setValue(suggestion.command + ' ');
    setShowSuggestions(false);
    textareaRef.current?.focus();
  }, []);

  const handleKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (showSuggestions && suggestions.length > 0) {
        if (e.key === 'ArrowDown') {
          e.preventDefault();
          setSelectedIndex((prev) => (prev + 1) % suggestions.length);
          return;
        }
        if (e.key === 'ArrowUp') {
          e.preventDefault();
          setSelectedIndex((prev) => (prev - 1 + suggestions.length) % suggestions.length);
          return;
        }
        if (e.key === 'Tab' || (e.key === 'Enter' && !e.shiftKey)) {
          e.preventDefault();
          applySuggestion(suggestions[selectedIndex]);
          return;
        }
        if (e.key === 'Escape') {
          e.preventDefault();
          setShowSuggestions(false);
          return;
        }
      }

      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleSubmit(e);
      }
    },
    [handleSubmit, showSuggestions, suggestions, selectedIndex, applySuggestion]
  );

  return (
    <form className="task-input" onSubmit={handleSubmit}>
      <div className="task-input-container">
        {showSuggestions && (
          <div className="command-suggestions">
            {suggestions.map((suggestion, index) => (
              <button
                key={suggestion.command}
                ref={(el) => {
                  suggestionRefs.current[index] = el;
                }}
                type="button"
                className={`suggestion-item ${index === selectedIndex ? 'selected' : ''}`}
                onClick={() => applySuggestion(suggestion)}
              >
                <span className="suggestion-command">{suggestion.command}</span>
                <span className="suggestion-description">{suggestion.description}</span>
              </button>
            ))}
          </div>
        )}
        <textarea
          ref={textareaRef}
          value={value}
          onChange={(e) => setValue(e.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Enter a task or type / for commands..."
          disabled={disabled}
          rows={3}
        />
      </div>
      <button type="submit" disabled={disabled || !value.trim()}>
        Send
      </button>
    </form>
  );
}

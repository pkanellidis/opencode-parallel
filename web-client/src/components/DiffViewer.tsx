import ReactDiffViewer, { DiffMethod } from 'react-diff-viewer-continued';
import { useMemo } from 'react';

interface DiffViewerProps {
  oldValue: string;
  newValue: string;
  filename?: string;
  splitView?: boolean;
}

const customStyles = {
  variables: {
    dark: {
      diffViewerBackground: '#0d1117',
      diffViewerColor: '#e6edf3',
      addedBackground: '#1a4721',
      addedColor: '#3fb950',
      removedBackground: '#6e2b2b',
      removedColor: '#f85149',
      wordAddedBackground: '#2ea04333',
      wordRemovedBackground: '#f8514966',
      addedGutterBackground: '#238636',
      removedGutterBackground: '#da3633',
      gutterBackground: '#161b22',
      gutterBackgroundDark: '#161b22',
      highlightBackground: '#fffbdd11',
      highlightGutterBackground: '#fffbdd11',
      codeFoldGutterBackground: '#21262d',
      codeFoldBackground: '#21262d',
      emptyLineBackground: '#161b22',
      codeFoldContentColor: '#8b949e',
    },
  },
  line: {
    padding: '4px 10px',
    fontFamily: "'JetBrains Mono', ui-monospace, monospace",
    fontSize: '13px',
  },
  gutter: {
    minWidth: '40px',
    padding: '0 8px',
    fontFamily: "'JetBrains Mono', ui-monospace, monospace",
    fontSize: '12px',
  },
  marker: {
    padding: '0 8px',
  },
  contentText: {
    fontFamily: "'JetBrains Mono', ui-monospace, monospace",
    fontSize: '13px',
    lineHeight: '1.5',
  },
  content: {
    width: '100%',
  },
  wordDiff: {
    padding: '2px 0',
  },
  diffContainer: {
    borderRadius: '6px',
    overflow: 'hidden',
  },
  diffRemoved: {
    backgroundColor: '#6e2b2b',
  },
  diffAdded: {
    backgroundColor: '#1a4721',
  },
};

export function DiffViewer({
  oldValue,
  newValue,
  filename,
  splitView = false,
}: DiffViewerProps) {
  const fileExtension = useMemo(() => {
    if (!filename) return undefined;
    const ext = filename.split('.').pop()?.toLowerCase();
    return ext;
  }, [filename]);

  const language = useMemo(() => {
    const langMap: Record<string, string> = {
      js: 'javascript',
      ts: 'typescript',
      tsx: 'typescript',
      jsx: 'javascript',
      py: 'python',
      rb: 'ruby',
      rs: 'rust',
      go: 'go',
      java: 'java',
      c: 'c',
      cpp: 'cpp',
      cs: 'csharp',
      php: 'php',
      json: 'json',
      yaml: 'yaml',
      yml: 'yaml',
      md: 'markdown',
      css: 'css',
      scss: 'scss',
      html: 'html',
      xml: 'xml',
      sql: 'sql',
      sh: 'bash',
      bash: 'bash',
    };
    return fileExtension ? langMap[fileExtension] : undefined;
  }, [fileExtension]);

  return (
    <div className="diff-viewer-container">
      {filename && (
        <div className="diff-viewer-header">
          <span className="diff-filename">{filename}</span>
          {language && <span className="diff-language">{language}</span>}
        </div>
      )}
      <div className="diff-viewer-content">
        <ReactDiffViewer
          oldValue={oldValue}
          newValue={newValue}
          splitView={splitView}
          useDarkTheme={true}
          compareMethod={DiffMethod.WORDS}
          styles={customStyles}
          showDiffOnly={false}
          extraLinesSurroundingDiff={3}
          hideLineNumbers={false}
        />
      </div>
    </div>
  );
}

interface FileDiffProps {
  filePath: string;
  oldContent?: string;
  newContent: string;
  isNewFile?: boolean;
}

export function FileDiff({ filePath, oldContent, newContent, isNewFile }: FileDiffProps) {
  return (
    <div className="file-diff">
      <div className="file-diff-header">
        <span className={`file-diff-badge ${isNewFile ? 'new' : 'modified'}`}>
          {isNewFile ? 'NEW' : 'MODIFIED'}
        </span>
        <span className="file-diff-path">{filePath}</span>
      </div>
      <DiffViewer
        oldValue={oldContent || ''}
        newValue={newContent}
        filename={filePath}
      />
    </div>
  );
}

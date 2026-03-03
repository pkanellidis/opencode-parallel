import { useEffect, useState } from 'react';
import { codeToHtml, type BundledLanguage } from 'shiki';

interface CodeBlockProps {
  code: string;
  language?: string;
  filename?: string;
  showLineNumbers?: boolean;
}

const LANGUAGE_MAP: Record<string, string> = {
  js: 'javascript',
  ts: 'typescript',
  tsx: 'tsx',
  jsx: 'jsx',
  py: 'python',
  rb: 'ruby',
  rs: 'rust',
  go: 'go',
  java: 'java',
  c: 'c',
  cpp: 'cpp',
  cs: 'csharp',
  php: 'php',
  swift: 'swift',
  kt: 'kotlin',
  scala: 'scala',
  sh: 'bash',
  bash: 'bash',
  zsh: 'bash',
  json: 'json',
  yaml: 'yaml',
  yml: 'yaml',
  toml: 'toml',
  xml: 'xml',
  html: 'html',
  css: 'css',
  scss: 'scss',
  less: 'less',
  md: 'markdown',
  markdown: 'markdown',
  sql: 'sql',
  graphql: 'graphql',
  dockerfile: 'dockerfile',
  makefile: 'makefile',
  txt: 'text',
};

function getLanguageFromFilename(filename: string): string {
  const ext = filename.split('.').pop()?.toLowerCase() || '';
  return LANGUAGE_MAP[ext] || 'text';
}

export function CodeBlock({
  code,
  language,
  filename,
  showLineNumbers = true,
}: CodeBlockProps) {
  const [html, setHtml] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);

  const effectiveLanguage = language
    ? (LANGUAGE_MAP[language] || language)
    : filename
    ? getLanguageFromFilename(filename)
    : 'text';

  useEffect(() => {
    let cancelled = false;

    async function highlight() {
      try {
        const result = await codeToHtml(code, {
          lang: effectiveLanguage as BundledLanguage,
          theme: 'github-dark',
        });
        if (!cancelled) {
          setHtml(result);
          setIsLoading(false);
        }
      } catch {
        if (!cancelled) {
          setHtml(`<pre><code>${escapeHtml(code)}</code></pre>`);
          setIsLoading(false);
        }
      }
    }

    highlight();

    return () => {
      cancelled = true;
    };
  }, [code, effectiveLanguage]);

  if (isLoading) {
    return (
      <div className="code-block loading">
        {filename && <div className="code-block-header">{filename}</div>}
        <pre className="code-block-content">
          <code>{code}</code>
        </pre>
      </div>
    );
  }

  return (
    <div className="code-block">
      {filename && <div className="code-block-header">{filename}</div>}
      <div
        className={`code-block-content ${showLineNumbers ? 'with-line-numbers' : ''}`}
        dangerouslySetInnerHTML={{ __html: html }}
      />
    </div>
  );
}

function escapeHtml(str: string): string {
  return str
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

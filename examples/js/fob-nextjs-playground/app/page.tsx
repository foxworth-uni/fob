'use client';

import { useState, useCallback } from 'react';

const defaultCode = `// Interactive Fob Playground
// Edit this code and click "Compile" to see the bundled output

export function greet(name: string): string {
  return \`Hello, \${name}!\`;
}

export function App() {
  const message = greet('Fob');
  return <div>{message}</div>;
}

console.log(greet('World'));
`;

interface Stats {
  duration: number;
  modules: number;
  chunks: number;
  size: number;
  cacheHitRate: number;
}

export default function PlaygroundPage() {
  const [code, setCode] = useState(defaultCode);
  const [output, setOutput] = useState('');
  const [error, setError] = useState('');
  const [stats, setStats] = useState<Stats | null>(null);
  const [isCompiling, setIsCompiling] = useState(false);

  const compile = useCallback(async () => {
    if (!code.trim() || isCompiling) return;

    setIsCompiling(true);
    setError('');

    try {
      const response = await fetch('/api/compile', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ code, filename: 'main.tsx' }),
      });

      const result = await response.json();

      if (result.error) {
        setError(result.error);
        setOutput('');
      } else {
        setOutput(result.output);
        setStats(result.stats);
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Network error');
    } finally {
      setIsCompiling(false);
    }
  }, [code, isCompiling]);

  const loadTemplate = useCallback(async (name: string) => {
    if (!name) return;

    try {
      const response = await fetch(`/api/templates?name=${name}`);
      const result = await response.json();

      if (result.content) {
        setCode(result.content);
      }
    } catch (err) {
      console.error('Failed to load template:', err);
    }
  }, []);

  const formatBytes = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };

  return (
    <div style={styles.container}>
      <header style={styles.header}>
        <div style={styles.logo}>
          <span style={styles.logoAccent}>fob</span> playground
          <span style={styles.badge}>Next.js</span>
        </div>
        <div style={styles.controls}>
          <select
            style={styles.select}
            onChange={(e) => loadTemplate(e.target.value)}
            defaultValue=""
          >
            <option value="">Load template...</option>
            <option value="react-app">React App</option>
            <option value="counter">Counter</option>
            <option value="hello">Hello World</option>
          </select>
          <button
            style={{
              ...styles.button,
              opacity: isCompiling ? 0.6 : 1,
            }}
            onClick={compile}
            disabled={isCompiling}
          >
            {isCompiling ? 'Compiling...' : 'Compile'}
          </button>
        </div>
      </header>

      <main style={styles.main}>
        <div style={styles.panel}>
          <div style={styles.panelHeader}>Input (TypeScript/JSX)</div>
          <textarea
            style={styles.textarea}
            value={code}
            onChange={(e) => setCode(e.target.value)}
            onKeyDown={(e) => {
              if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
                e.preventDefault();
                compile();
              }
            }}
            placeholder="// Write your code here..."
            spellCheck={false}
          />
        </div>

        <div style={styles.panel}>
          <div style={styles.panelHeader}>Output (Bundled)</div>
          <div style={styles.outputContainer}>
            {error ? (
              <div style={styles.error}>{error}</div>
            ) : (
              <pre style={styles.pre}>{output}</pre>
            )}
          </div>
          {stats && (
            <div style={styles.statsBar}>
              <div style={styles.stat}>
                <span style={styles.statLabel}>Duration:</span>
                <span
                  style={{
                    ...styles.statValue,
                    color:
                      stats.duration < 100
                        ? '#4ade80'
                        : stats.duration < 500
                          ? '#e4e4e4'
                          : '#fbbf24',
                  }}
                >
                  {stats.duration}ms
                </span>
              </div>
              <div style={styles.stat}>
                <span style={styles.statLabel}>Modules:</span>
                <span style={styles.statValue}>{stats.modules}</span>
              </div>
              <div style={styles.stat}>
                <span style={styles.statLabel}>Size:</span>
                <span style={styles.statValue}>{formatBytes(stats.size)}</span>
              </div>
              <div style={styles.stat}>
                <span style={styles.statLabel}>Cache:</span>
                <span
                  style={{
                    ...styles.statValue,
                    color: stats.cacheHitRate > 0.5 ? '#4ade80' : '#e4e4e4',
                  }}
                >
                  {Math.round(stats.cacheHitRate * 100)}%
                </span>
              </div>
            </div>
          )}
        </div>
      </main>
    </div>
  );
}

const styles: Record<string, React.CSSProperties> = {
  container: {
    minHeight: '100vh',
    background: '#1a1a2e',
    color: '#e4e4e4',
    fontFamily: "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
  },
  header: {
    background: '#16213e',
    borderBottom: '1px solid #0f3460',
    padding: '16px 24px',
    display: 'flex',
    alignItems: 'center',
    justifyContent: 'space-between',
  },
  logo: {
    fontSize: '1.5rem',
    fontWeight: 400,
    display: 'flex',
    alignItems: 'center',
    gap: '8px',
  },
  logoAccent: {
    color: '#e94560',
    fontWeight: 700,
  },
  badge: {
    background: '#0f3460',
    color: '#888',
    fontSize: '0.7rem',
    padding: '4px 8px',
    borderRadius: '4px',
    marginLeft: '8px',
  },
  controls: {
    display: 'flex',
    gap: '12px',
    alignItems: 'center',
  },
  select: {
    background: '#16213e',
    color: '#e4e4e4',
    border: '1px solid #0f3460',
    padding: '8px 12px',
    borderRadius: '6px',
    fontSize: '13px',
    cursor: 'pointer',
  },
  button: {
    background: '#e94560',
    color: 'white',
    border: 'none',
    padding: '10px 20px',
    borderRadius: '6px',
    fontSize: '14px',
    fontWeight: 600,
    cursor: 'pointer',
  },
  main: {
    display: 'grid',
    gridTemplateColumns: '1fr 1fr',
    height: 'calc(100vh - 65px)',
  },
  panel: {
    display: 'flex',
    flexDirection: 'column',
    borderRight: '1px solid #0f3460',
  },
  panelHeader: {
    background: '#16213e',
    padding: '12px 16px',
    borderBottom: '1px solid #0f3460',
    fontSize: '13px',
    fontWeight: 600,
    color: '#888',
    textTransform: 'uppercase',
    letterSpacing: '0.5px',
  },
  textarea: {
    flex: 1,
    width: '100%',
    background: '#1a1a2e',
    color: '#e4e4e4',
    border: 'none',
    padding: '16px',
    fontFamily: "'SF Mono', 'Fira Code', monospace",
    fontSize: '14px',
    lineHeight: 1.6,
    resize: 'none',
    outline: 'none',
  },
  outputContainer: {
    flex: 1,
    overflow: 'auto',
  },
  pre: {
    padding: '16px',
    margin: 0,
    fontFamily: "'SF Mono', 'Fira Code', monospace",
    fontSize: '13px',
    lineHeight: 1.6,
    whiteSpace: 'pre-wrap',
    wordBreak: 'break-word',
  },
  error: {
    color: '#e94560',
    background: 'rgba(233, 69, 96, 0.1)',
    padding: '16px',
    borderRadius: '4px',
    margin: '16px',
  },
  statsBar: {
    background: '#16213e',
    borderTop: '1px solid #0f3460',
    padding: '12px 16px',
    display: 'flex',
    gap: '24px',
    fontSize: '13px',
  },
  stat: {
    display: 'flex',
    alignItems: 'center',
    gap: '6px',
  },
  statLabel: {
    color: '#888',
  },
  statValue: {
    fontWeight: 600,
    fontFamily: "'SF Mono', monospace",
    color: '#e4e4e4',
  },
};

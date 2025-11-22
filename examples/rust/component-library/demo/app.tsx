import React from 'react';
import { createRoot } from 'react-dom/client';
import { Button, Card, Badge } from '../components';

function Demo() {
  const [count, setCount] = React.useState(0);

  return (
    <>
      <div className="demo-section">
        <h2>Button Component</h2>
        <div className="component-grid">
          <Button variant="primary" onClick={() => setCount(count + 1)}>
            Primary Button
          </Button>
          <Button variant="secondary" onClick={() => setCount(count - 1)}>
            Secondary Button
          </Button>
          <Button variant="danger" onClick={() => setCount(0)}>
            Danger Button
          </Button>
        </div>
        <p style={{ marginTop: '1rem', fontSize: '1.1rem', fontWeight: 'bold' }}>
          Click count: {count}
        </p>
        <details style={{ marginTop: '1rem' }}>
          <summary style={{ cursor: 'pointer', fontWeight: 500 }}>Usage Example</summary>
          <pre
            style={{
              background: '#f5f5f5',
              padding: '1rem',
              borderRadius: '4px',
              overflow: 'auto',
            }}
          >
            {`import { Button } from 'my-component-lib';

<Button variant="primary" onClick={() => alert('Clicked!')}>
  Click Me
</Button>`}
          </pre>
        </details>
      </div>

      <div className="demo-section">
        <h2>Card Component</h2>
        <div className="component-grid">
          <Card title="Welcome">
            <p>This is a simple card component with a title and body content.</p>
          </Card>
          <Card title="Features">
            <ul style={{ margin: 0, paddingLeft: '1.5rem' }}>
              <li>TypeScript support</li>
              <li>Tree-shakeable</li>
              <li>Zero dependencies</li>
            </ul>
          </Card>
          <Card title="Stats">
            <p style={{ margin: '0.5rem 0' }}>Modules: 3</p>
            <p style={{ margin: '0.5rem 0' }}>Size: ~2KB</p>
            <p style={{ margin: '0.5rem 0' }}>Build time: &lt;1s</p>
          </Card>
        </div>
        <details style={{ marginTop: '1rem' }}>
          <summary style={{ cursor: 'pointer', fontWeight: 500 }}>Usage Example</summary>
          <pre
            style={{
              background: '#f5f5f5',
              padding: '1rem',
              borderRadius: '4px',
              overflow: 'auto',
            }}
          >
            {`import { Card } from 'my-component-lib';

<Card title="My Card">
  <p>Card content goes here</p>
</Card>`}
          </pre>
        </details>
      </div>

      <div className="demo-section">
        <h2>Badge Component</h2>
        <div className="badge-container">
          <Badge variant="success">Success</Badge>
          <Badge variant="warning">Warning</Badge>
          <Badge variant="error">Error</Badge>
          <Badge variant="info">Info</Badge>
        </div>
        <details style={{ marginTop: '1rem' }}>
          <summary style={{ cursor: 'pointer', fontWeight: 500 }}>Usage Example</summary>
          <pre
            style={{
              background: '#f5f5f5',
              padding: '1rem',
              borderRadius: '4px',
              overflow: 'auto',
            }}
          >
            {`import { Badge } from 'my-component-lib';

<Badge variant="success">New</Badge>
<Badge variant="warning">Beta</Badge>`}
          </pre>
        </details>
      </div>

      <div className="demo-section">
        <h2>📦 Build Information</h2>
        <p>This demo shows the component library built with fob.</p>
        <ul>
          <li>
            <strong>Library mode:</strong> Externalizes React as peer dependency
          </li>
          <li>
            <strong>Tree-shakeable:</strong> Import only what you need
          </li>
          <li>
            <strong>TypeScript:</strong> Full type definitions included
          </li>
          <li>
            <strong>Source maps:</strong> Easy debugging in development
          </li>
          <li>
            <strong>ESM format:</strong> Modern ES modules for optimal bundling
          </li>
        </ul>

        <h3 style={{ marginTop: '1.5rem', color: '#555' }}>Tree-Shaking Example</h3>
        <p>Import only the components you need:</p>
        <pre
          style={{ background: '#f5f5f5', padding: '1rem', borderRadius: '4px', overflow: 'auto' }}
        >
          {`// Import everything (not recommended for production)
import { Button, Card, Badge } from 'my-component-lib';

// Import only what you need (tree-shakeable)
import { Button } from 'my-component-lib';`}
        </pre>
      </div>
    </>
  );
}

const root = createRoot(document.getElementById('root')!);
root.render(<Demo />);

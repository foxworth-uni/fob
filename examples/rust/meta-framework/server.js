import { serve } from '@hono/node-server';
import { Hono } from 'hono';

const app = new Hono();

// Route metadata - maps paths to their info
const routeInfo = {
  '/': {
    title: 'Home',
    description: 'Welcome to the meta-framework example',
  },
  '/about': {
    title: 'About',
    description: 'Learn about this meta-framework example',
  },
};

// HTML template - demonstrates the framework concept
// In a real framework, you'd do SSR here, but this example shows the build process
function htmlTemplate(path) {
  const info = routeInfo[path] || { title: '404', description: 'Page not found' };
  
  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>${info.title} - Meta-Framework Example</title>
  <meta name="description" content="${info.description}">
  <style>
    body {
      font-family: system-ui, -apple-system, sans-serif;
      max-width: 800px;
      margin: 40px auto;
      padding: 0 20px;
      line-height: 1.6;
      color: #333;
    }
    nav {
      margin-bottom: 2rem;
      padding-bottom: 1rem;
      border-bottom: 2px solid #eee;
    }
    nav a {
      margin-right: 1rem;
      color: #0066cc;
      text-decoration: none;
      font-weight: 500;
    }
    nav a:hover {
      text-decoration: underline;
    }
    h1 {
      color: #111;
      margin-bottom: 1rem;
    }
    p {
      margin-bottom: 0.75rem;
    }
    .footer {
      margin-top: 3rem;
      padding-top: 1rem;
      border-top: 1px solid #eee;
      color: #666;
      font-size: 0.9rem;
    }
    .demo-note {
      background: #f0f7ff;
      border-left: 4px solid #0066cc;
      padding: 1rem;
      margin: 2rem 0;
    }
  </style>
</head>
<body>
  <nav>
    <a href="/">Home</a>
    <a href="/about">About</a>
  </nav>
  
  <div class="demo-note">
    <strong>📦 Meta-Framework Build Demo</strong>
    <p>This example demonstrates how fob discovers routes and bundles them with code splitting.</p>
    <p>Current route: <code>${path}</code></p>
    <p>Bundle: <code>/dist${path === '/' ? '/index' : path}.js</code></p>
  </div>

  <div id="root">
    <h1>${info.title}</h1>
    <p>${info.description}</p>
    
    ${path === '/' ? `
      <h2>What This Example Shows</h2>
      <ul>
        <li><strong>File-based routing</strong> - Routes discovered from app/routes/</li>
        <li><strong>Code splitting</strong> - Each route is a separate bundle</li>
        <li><strong>Shared chunks</strong> - React extracted into jsx-runtime chunk</li>
        <li><strong>Build optimization</strong> - Minified output with source maps</li>
      </ul>
      
      <h2>Generated Bundles</h2>
      <ul>
        <li><code>index.js</code> - Home route (this page)</li>
        <li><code>about.js</code> - About route</li>
        <li><code>jsx-runtime-*.js</code> - Shared React runtime</li>
      </ul>
    ` : ''}
    
    ${path === '/about' ? `
      <h2>Meta-Framework Concepts</h2>
      <p>This example demonstrates the core bundling pattern used by frameworks like Next.js and Remix:</p>
      <ol>
        <li>Scan a directory structure to discover routes</li>
        <li>Bundle each route as a separate entry point</li>
        <li>Extract shared code into common chunks</li>
        <li>Generate optimized output for production</li>
      </ol>
      
      <p>The Rust code in <code>src/main.rs</code> shows how to use fob's API to implement this pattern.</p>
    ` : ''}
  </div>
  
  <div class="footer">
    <p>🚀 Powered by fob - Meta-framework build example</p>
    <p>Check <code>dist/</code> to see the generated bundles</p>
  </div>
</body>
</html>`;
}

// Route handler
app.get('*', (c) => {
  const path = c.req.path;

  if (!routeInfo[path]) {
    return c.html(htmlTemplate(path), 404);
  }

  return c.html(htmlTemplate(path));
});

const port = 3000;
console.log(`🚀 Server running at http://localhost:${port}`);
console.log(`📁 Routes available:`);
Object.keys(routeInfo).forEach(route => {
  console.log(`   • http://localhost:${port}${route}`);
});

serve({
  fetch: app.fetch,
  port,
});


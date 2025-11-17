import { serve } from '@hono/node-server';
import { Hono } from 'hono';
import { readFileSync } from 'fs';
import { join, dirname } from 'path';
import { fileURLToPath } from 'url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const app = new Hono();

// Serve demo HTML
app.get('/', (c) => {
  try {
    const html = readFileSync(join(__dirname, 'demo/index.html'), 'utf-8');
    return c.html(html);
  } catch (error) {
    return c.text('Demo not found. Make sure demo/index.html exists.', 404);
  }
});

// Serve demo built files (bundled by fob)
app.get('/demo/dist/*', (c) => {
  try {
    const path = c.req.path.slice(11); // Remove '/demo/dist/'
    const filePath = join(__dirname, 'demo/dist', path);
    const file = readFileSync(filePath, 'utf-8');
    
    const ext = path.split('.').pop();
    const contentType = 
      ext === 'js' ? 'application/javascript' :
      ext === 'map' ? 'application/json' :
      'text/plain';
    
    return c.body(file, 200, { 'Content-Type': contentType });
  } catch (error) {
    return c.text(`Demo file not found: ${c.req.path}. Did you run 'cargo run' to build?`, 404);
  }
});

// All other routes serve the built library (optional, for testing imports)
app.get('/dist/*', (c) => {
  try {
    const path = c.req.path.slice(6); // Remove '/dist/'
    const filePath = join(__dirname, 'dist', path);
    const file = readFileSync(filePath, 'utf-8');
    
    const ext = path.split('.').pop();
    const contentType = 
      ext === 'js' ? 'application/javascript' :
      ext === 'map' ? 'application/json' :
      ext === 'ts' ? 'application/typescript' :
      'text/plain';
    
    return c.body(file, 200, { 'Content-Type': contentType });
  } catch (error) {
    return c.text(`Library file not found: ${c.req.path}`, 404);
  }
});

const port = 3001;
console.log(`🎨 Component Library Demo`);
console.log(`📦 Server running at http://localhost:${port}`);
console.log(`\nComponents available:`);
console.log(`   • Button (primary, secondary, danger variants)`);
console.log(`   • Card (title + content)`);
console.log(`   • Badge (success, warning, error, info variants)`);

serve({
  fetch: app.fetch,
  port,
});


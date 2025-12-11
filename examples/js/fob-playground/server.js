/**
 * Fob Playground Server
 *
 * A dev tool example showing how to use Fob for on-demand compilation.
 * Demonstrates: virtual files, HTTP API, SSE live updates, build stats.
 */

import { serve } from '@hono/node-server';
import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { serveStatic } from '@hono/node-server/serve-static';
import { streamSSE } from 'hono/streaming';
import { EventEmitter } from 'node:events';
import { readFile } from 'node:fs/promises';
import { fileURLToPath } from 'node:url';
import { dirname, join } from 'node:path';

import { bundle } from './src/bundler.js';

const __dirname = dirname(fileURLToPath(import.meta.url));

const app = new Hono();
const buildEmitter = new EventEmitter();

// Enable CORS for development
app.use('/*', cors());

// Serve static files from public/
app.use('/public/*', serveStatic({ root: './' }));

// Build history for stats dashboard
const buildHistory = [];
const MAX_HISTORY = 50;

/**
 * POST /api/compile
 * Compile user code on-demand using virtual files
 */
app.post('/api/compile', async (c) => {
  const startTime = performance.now();

  try {
    const { code, filename = 'main.tsx' } = await c.req.json();

    if (!code || typeof code !== 'string') {
      return c.json({ error: 'Missing or invalid "code" field' }, 400);
    }

    const result = await bundle(code, filename);

    const buildInfo = {
      timestamp: Date.now(),
      duration: Math.round(performance.now() - startTime),
      modules: result.stats.totalModules,
      chunks: result.stats.totalChunks,
      size: result.stats.totalSize,
      cacheHitRate: result.stats.cacheHitRate,
      success: true,
    };

    // Track build history
    buildHistory.push(buildInfo);
    if (buildHistory.length > MAX_HISTORY) {
      buildHistory.shift();
    }

    // Emit for SSE subscribers
    buildEmitter.emit('build', buildInfo);

    return c.json({
      output: result.chunks[0]?.code || '',
      stats: {
        duration: buildInfo.duration,
        modules: buildInfo.modules,
        chunks: buildInfo.chunks,
        size: buildInfo.size,
        cacheHitRate: buildInfo.cacheHitRate,
      },
    });
  } catch (error) {
    const buildInfo = {
      timestamp: Date.now(),
      duration: Math.round(performance.now() - startTime),
      success: false,
      error: error.message,
    };

    buildHistory.push(buildInfo);
    buildEmitter.emit('build', buildInfo);

    return c.json(
      {
        error: error.message,
        details: error.details || null,
      },
      500
    );
  }
});

/**
 * GET /api/sse
 * Server-sent events for live build updates
 */
app.get('/api/sse', (c) => {
  return streamSSE(c, async (stream) => {
    const onBuild = (data) => {
      stream.writeSSE({ data: JSON.stringify(data), event: 'build' });
    };

    buildEmitter.on('build', onBuild);

    // Send initial connection message
    stream.writeSSE({ data: JSON.stringify({ connected: true }), event: 'connected' });

    // Keep connection alive
    const keepAlive = setInterval(() => {
      stream.writeSSE({ data: '', event: 'ping' });
    }, 30000);

    // Cleanup on disconnect
    stream.onAbort(() => {
      clearInterval(keepAlive);
      buildEmitter.off('build', onBuild);
    });

    // Keep the stream open
    await new Promise(() => {});
  });
});

/**
 * GET /api/stats
 * Get build history and aggregate statistics
 */
app.get('/api/stats', (c) => {
  const successful = buildHistory.filter((b) => b.success);
  const avgDuration =
    successful.length > 0
      ? successful.reduce((sum, b) => sum + b.duration, 0) / successful.length
      : 0;

  const avgCacheHitRate =
    successful.length > 0
      ? successful.reduce((sum, b) => sum + (b.cacheHitRate || 0), 0) / successful.length
      : 0;

  return c.json({
    totalBuilds: buildHistory.length,
    successfulBuilds: successful.length,
    failedBuilds: buildHistory.length - successful.length,
    avgDuration: Math.round(avgDuration),
    avgCacheHitRate: Math.round(avgCacheHitRate * 100),
    recentBuilds: buildHistory.slice(-10),
  });
});

/**
 * GET /api/templates/:name
 * Get starter templates
 */
app.get('/api/templates/:name', async (c) => {
  const name = c.req.param('name');
  const templatePath = join(__dirname, 'templates', `${name}.tsx`);

  try {
    const content = await readFile(templatePath, 'utf-8');
    return c.json({ content, filename: `${name}.tsx` });
  } catch {
    return c.json({ error: `Template "${name}" not found` }, 404);
  }
});

/**
 * GET /
 * Serve the playground UI
 */
app.get('/', async (c) => {
  const html = await readFile(join(__dirname, 'public', 'index.html'), 'utf-8');
  return c.html(html);
});

// Start server
const port = process.env.PORT || 3000;

serve({ fetch: app.fetch, port }, (info) => {
  console.log(`
  Fob Playground Server

  Local:   http://localhost:${info.port}
  API:     http://localhost:${info.port}/api/compile
  SSE:     http://localhost:${info.port}/api/sse
  Stats:   http://localhost:${info.port}/api/stats

  Ready to compile code on-demand!
  `);
});

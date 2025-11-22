import { createRouter } from './router';

// Simple server implementation for the meta-framework
// This demonstrates how framework code can import from path aliases

export function createServer() {
  const router = createRouter();

  // Register default routes
  router.add('/', () => '<h1>Welcome</h1>');

  return {
    fetch(request: Request) {
      const url = new URL(request.url);
      const handler = router.match(url.pathname);

      if (handler) {
        return new Response(handler(), {
          headers: { 'Content-Type': 'text/html' },
        });
      }

      return new Response('Not found', { status: 404 });
    },
  };
}

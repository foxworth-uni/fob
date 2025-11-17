/**
 * Edge runtime entry point - uses only Web APIs
 * No Node.js built-ins allowed
 */

import { handleRequest } from './api-handler.js';
import { hashData } from './web-crypto.js';

export async function GET(request) {
  const url = new URL(request.url);
  const name = url.searchParams.get('name') || 'World';

  const hash = await hashData(name);

  return handleRequest({
    greeting: `Hello, ${name}!`,
    hash,
    timestamp: Date.now(),
  });
}

export const runtime = 'edge';

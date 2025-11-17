/**
 * Edge API handler using Web APIs only
 */

export function handleRequest(data) {
  const response = new Response(JSON.stringify(data), {
    status: 200,
    headers: new Headers({
      'Content-Type': 'application/json',
      'Cache-Control': 'public, max-age=3600',
    }),
  });

  return response;
}

export async function handleError(error) {
  return new Response(JSON.stringify({ error: error.message }), {
    status: 500,
    headers: { 'Content-Type': 'application/json' },
  });
}

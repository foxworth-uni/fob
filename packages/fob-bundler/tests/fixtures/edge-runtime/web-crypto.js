/**
 * Web Crypto API usage (not Node.js crypto)
 * Available in edge runtimes
 */

export async function hashData(data) {
  const encoder = new TextEncoder();
  const dataBuffer = encoder.encode(data);

  const hashBuffer = await crypto.subtle.digest('SHA-256', dataBuffer);
  const hashArray = Array.from(new Uint8Array(hashBuffer));
  const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');

  return hashHex;
}

export function randomId() {
  return crypto.randomUUID();
}

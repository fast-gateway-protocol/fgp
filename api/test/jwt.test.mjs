import assert from 'node:assert/strict';
import { test } from 'node:test';
import { sign, verify } from '../dist/lib/jwt.js';

const encoder = new TextEncoder();

function base64UrlEncode(data) {
  const buffer = data instanceof ArrayBuffer ? Buffer.from(data) : Buffer.from(data);
  return buffer
    .toString('base64')
    .replace(/\+/g, '-')
    .replace(/\//g, '_')
    .replace(/=+$/, '');
}

async function signToken(payload, secret) {
  const header = { alg: 'HS256', typ: 'JWT' };
  const headerB64 = base64UrlEncode(encoder.encode(JSON.stringify(header)));
  const payloadB64 = base64UrlEncode(encoder.encode(JSON.stringify(payload)));
  const message = `${headerB64}.${payloadB64}`;

  const key = await crypto.subtle.importKey(
    'raw',
    encoder.encode(secret),
    { name: 'HMAC', hash: 'SHA-256' },
    false,
    ['sign', 'verify']
  );
  const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(message));
  return `${message}.${base64UrlEncode(signature)}`;
}

test('sign + verify roundtrip', async () => {
  const token = await sign(
    { sub: 'user_123', email: 'user@example.com', role: 'member' },
    'secret'
  );

  const payload = await verify(token, 'secret');
  assert.equal(payload.sub, 'user_123');
  assert.equal(payload.email, 'user@example.com');
  assert.equal(payload.role, 'member');
  assert.ok(payload.iat);
  assert.ok(payload.exp);
});

test('verify rejects tampered token', async () => {
  const token = await sign(
    { sub: 'user_456', email: 'user2@example.com', role: 'member' },
    'secret'
  );

  const parts = token.split('.');
  const tampered = `${parts[0]}.${parts[1].slice(0, -1)}A.${parts[2]}`;

  await assert.rejects(() => verify(tampered, 'secret'), /Invalid signature/);
});

test('verify rejects expired token', async () => {
  const now = Math.floor(Date.now() / 1000);
  const token = await signToken(
    {
      sub: 'user_789',
      email: 'user3@example.com',
      role: 'member',
      iat: now - 3600,
      exp: now - 60,
    },
    'secret'
  );

  await assert.rejects(() => verify(token, 'secret'), /Token expired/);
});

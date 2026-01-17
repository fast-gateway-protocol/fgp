// Simple JWT implementation using Web Crypto API
// For production, consider using a library like jose
const encoder = new TextEncoder();
const decoder = new TextDecoder();
async function createHmacKey(secret) {
    return crypto.subtle.importKey('raw', encoder.encode(secret), { name: 'HMAC', hash: 'SHA-256' }, false, ['sign', 'verify']);
}
function base64UrlEncode(data) {
    const bytes = data instanceof ArrayBuffer ? new Uint8Array(data) : data;
    let binary = '';
    for (let i = 0; i < bytes.length; i++) {
        binary += String.fromCharCode(bytes[i]);
    }
    return btoa(binary).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}
function base64UrlDecode(str) {
    str = str.replace(/-/g, '+').replace(/_/g, '/');
    while (str.length % 4)
        str += '=';
    const binary = atob(str);
    const bytes = new Uint8Array(binary.length);
    for (let i = 0; i < binary.length; i++) {
        bytes[i] = binary.charCodeAt(i);
    }
    return bytes;
}
export async function sign(payload, secret) {
    const header = { alg: 'HS256', typ: 'JWT' };
    const now = Math.floor(Date.now() / 1000);
    const fullPayload = {
        ...payload,
        iat: now,
        exp: now + 60 * 60 * 24 * 7, // 7 days
    };
    const headerB64 = base64UrlEncode(encoder.encode(JSON.stringify(header)));
    const payloadB64 = base64UrlEncode(encoder.encode(JSON.stringify(fullPayload)));
    const message = `${headerB64}.${payloadB64}`;
    const key = await createHmacKey(secret);
    const signature = await crypto.subtle.sign('HMAC', key, encoder.encode(message));
    return `${message}.${base64UrlEncode(signature)}`;
}
export async function verify(token, secret) {
    const parts = token.split('.');
    if (parts.length !== 3) {
        throw new Error('Invalid token format');
    }
    const [headerB64, payloadB64, signatureB64] = parts;
    const message = `${headerB64}.${payloadB64}`;
    const key = await createHmacKey(secret);
    const signature = base64UrlDecode(signatureB64);
    const valid = await crypto.subtle.verify('HMAC', key, signature.buffer, encoder.encode(message));
    if (!valid) {
        throw new Error('Invalid signature');
    }
    const payload = JSON.parse(decoder.decode(base64UrlDecode(payloadB64)));
    if (payload.exp && payload.exp < Math.floor(Date.now() / 1000)) {
        throw new Error('Token expired');
    }
    return payload;
}
//# sourceMappingURL=jwt.js.map
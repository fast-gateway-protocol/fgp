import { Hono } from 'hono';
import { eq, and } from 'drizzle-orm';
import { db, purchases, skills, skillVersions, licenseValidations } from '../db';

export const licensesRoutes = new Hono();

// Validate license (called by CLI)
licensesRoutes.post('/validate', async (c) => {
  const body = await c.req.json();
  const { license_key, machine_fingerprint, skill_name } = body;

  if (!license_key || !machine_fingerprint || !skill_name) {
    return c.json({ valid: false, error: 'Missing required fields' }, 400);
  }

  // Find purchase by license key
  const purchase = await db
    .select({
      id: purchases.id,
      userId: purchases.userId,
      skillId: purchases.skillId,
      skillVersionId: purchases.skillVersionId,
      licenseStatus: purchases.licenseStatus,
      machineFingerprints: purchases.machineFingerprints,
    })
    .from(purchases)
    .where(eq(purchases.licenseKey, license_key))
    .limit(1);

  if (purchase.length === 0) {
    await logValidation(null, machine_fingerprint, c, 'invalid');
    return c.json({ valid: false, error: 'Invalid license key' }, 401);
  }

  const p = purchase[0];

  // Check license status
  if (p.licenseStatus !== 'active') {
    await logValidation(p.id, machine_fingerprint, c, 'invalid');
    return c.json({ valid: false, error: `License ${p.licenseStatus}` }, 401);
  }

  // Verify skill matches
  const skill = await db
    .select({ name: skills.name })
    .from(skills)
    .where(eq(skills.id, p.skillId))
    .limit(1);

  if (skill.length === 0 || skill[0].name !== skill_name) {
    await logValidation(p.id, machine_fingerprint, c, 'invalid');
    return c.json({ valid: false, error: 'License not valid for this skill' }, 401);
  }

  // Check machine limit (max 3)
  const fingerprints = (p.machineFingerprints as string[]) || [];
  const MACHINE_LIMIT = 3;

  if (!fingerprints.includes(machine_fingerprint)) {
    if (fingerprints.length >= MACHINE_LIMIT) {
      await logValidation(p.id, machine_fingerprint, c, 'rate_limited');
      return c.json({
        valid: false,
        error: `Machine limit reached (${MACHINE_LIMIT} machines)`,
        machines: fingerprints.length,
        limit: MACHINE_LIMIT,
      }, 403);
    }

    // Add new machine
    fingerprints.push(machine_fingerprint);
    await db
      .update(purchases)
      .set({ machineFingerprints: fingerprints })
      .where(eq(purchases.id, p.id));
  }

  // Log successful validation
  await logValidation(p.id, machine_fingerprint, c, 'valid');

  return c.json({
    valid: true,
    license_key,
    skill_name,
    machines: fingerprints.length,
    limit: MACHINE_LIMIT,
    expires_at: null, // One-time purchase, no expiry
  });
});

// Get license details (authenticated)
licensesRoutes.get('/:key', async (c) => {
  const key = c.req.param('key');

  const purchase = await db
    .select({
      id: purchases.id,
      licenseKey: purchases.licenseKey,
      licenseStatus: purchases.licenseStatus,
      machineFingerprints: purchases.machineFingerprints,
      purchasedAt: purchases.purchasedAt,
      skill: {
        name: skills.name,
        slug: skills.slug,
        version: skills.version,
      },
    })
    .from(purchases)
    .leftJoin(skills, eq(purchases.skillId, skills.id))
    .where(eq(purchases.licenseKey, key))
    .limit(1);

  if (purchase.length === 0) {
    return c.json({ error: 'License not found' }, 404);
  }

  return c.json({
    license_key: purchase[0].licenseKey,
    status: purchase[0].licenseStatus,
    machines: (purchase[0].machineFingerprints as string[])?.length || 0,
    machine_limit: 3,
    purchased_at: purchase[0].purchasedAt,
    skill: purchase[0].skill,
  });
});

// Download package (validated license required)
licensesRoutes.get('/:key/download', async (c) => {
  const key = c.req.param('key');
  const machineFingerprint = c.req.header('X-Machine-Fingerprint');

  if (!machineFingerprint) {
    return c.json({ error: 'Machine fingerprint required' }, 400);
  }

  const purchase = await db
    .select({
      id: purchases.id,
      skillVersionId: purchases.skillVersionId,
      licenseStatus: purchases.licenseStatus,
      machineFingerprints: purchases.machineFingerprints,
    })
    .from(purchases)
    .where(eq(purchases.licenseKey, key))
    .limit(1);

  if (purchase.length === 0) {
    return c.json({ error: 'License not found' }, 404);
  }

  const p = purchase[0];

  if (p.licenseStatus !== 'active') {
    return c.json({ error: `License ${p.licenseStatus}` }, 401);
  }

  // Verify machine is registered
  const fingerprints = (p.machineFingerprints as string[]) || [];
  if (!fingerprints.includes(machineFingerprint)) {
    return c.json({ error: 'Machine not registered for this license' }, 403);
  }

  // Get version info
  const version = await db
    .select({
      archiveUrl: skillVersions.archiveUrl,
      archiveChecksum: skillVersions.archiveChecksum,
      archiveSizeBytes: skillVersions.archiveSizeBytes,
      version: skillVersions.version,
    })
    .from(skillVersions)
    .where(eq(skillVersions.id, p.skillVersionId))
    .limit(1);

  if (version.length === 0 || !version[0].archiveUrl) {
    return c.json({ error: 'Package not available' }, 404);
  }

  // Generate time-limited download URL with decryption key
  // In production, this would be a signed URL to your CDN
  const decryptionKey = await generateDecryptionKey(key, machineFingerprint);

  return c.json({
    archive_url: version[0].archiveUrl,
    checksum_sha256: version[0].archiveChecksum,
    size_bytes: version[0].archiveSizeBytes,
    version: version[0].version,
    decryption_key: decryptionKey,
    expires_at: new Date(Date.now() + 60 * 60 * 1000).toISOString(), // 1 hour
  });
});

// Helper to log validation attempts
async function logValidation(
  purchaseId: string | null,
  fingerprint: string,
  c: any,
  result: string
) {
  if (!purchaseId) return;

  await db.insert(licenseValidations).values({
    purchaseId,
    machineFingerprint: fingerprint,
    ipAddress: c.req.header('CF-Connecting-IP') || c.req.header('X-Forwarded-For'),
    userAgent: c.req.header('User-Agent'),
    result,
  });
}

// Generate decryption key (simplified - in production use proper key derivation)
async function generateDecryptionKey(licenseKey: string, fingerprint: string): Promise<string> {
  const encoder = new TextEncoder();
  const data = encoder.encode(`${licenseKey}:${fingerprint}:${process.env.ENCRYPTION_SECRET}`);
  const hash = await crypto.subtle.digest('SHA-256', data);
  const hashArray = Array.from(new Uint8Array(hash));
  return hashArray.map((b) => b.toString(16).padStart(2, '0')).join('');
}

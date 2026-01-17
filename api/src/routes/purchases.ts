import { Hono } from 'hono';
import { getCookie } from 'hono/cookie';
import { eq, and } from 'drizzle-orm';
import Stripe from 'stripe';
import { db, purchases, skills, skillVersions, users } from '../db';
import { verify } from '../lib/jwt';

export const purchasesRoutes = new Hono();

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);

const JWT_SECRET = process.env.JWT_SECRET!;
const FRONTEND_URL = process.env.FRONTEND_URL || 'https://fgp.dev';

// Revenue split by tier
const REVENUE_SPLITS = {
  free: { platform: 0, seller: 100, stripe: 0 },
  community: { platform: 30, seller: 65, stripe: 5 },
  verified: { platform: 25, seller: 70, stripe: 5 },
  pro: { platform: 20, seller: 75, stripe: 5 },
};

// Create checkout session
purchasesRoutes.post('/:slug/checkout', async (c) => {
  const slug = c.req.param('slug');
  const token = getCookie(c, 'auth_token');

  if (!token) {
    return c.json({ error: 'Authentication required' }, 401);
  }

  let userId: string;
  try {
    const payload = await verify(token, JWT_SECRET);
    userId = payload.sub;
  } catch {
    return c.json({ error: 'Invalid token' }, 401);
  }

  // Get skill info
  const skill = await db
    .select()
    .from(skills)
    .where(and(eq(skills.slug, slug), eq(skills.status, 'active')))
    .limit(1);

  if (skill.length === 0) {
    return c.json({ error: 'Skill not found' }, 404);
  }

  const s = skill[0];

  if (s.priceCents === 0) {
    return c.json({ error: 'Skill is free' }, 400);
  }

  // Check if already purchased
  const existing = await db
    .select()
    .from(purchases)
    .where(and(eq(purchases.userId, userId), eq(purchases.skillId, s.id)))
    .limit(1);

  if (existing.length > 0) {
    return c.json({ error: 'Already purchased', license_key: existing[0].licenseKey }, 400);
  }

  // Get user's Stripe customer ID
  const user = await db.select().from(users).where(eq(users.id, userId)).limit(1);
  if (user.length === 0) {
    return c.json({ error: 'User not found' }, 404);
  }

  let stripeCustomerId = user[0].stripeCustomerId;

  // Create Stripe customer if needed
  if (!stripeCustomerId) {
    const customer = await stripe.customers.create({
      email: user[0].email,
      name: user[0].name || undefined,
      metadata: {
        fgp_user_id: userId,
        github_username: user[0].githubUsername || '',
      },
    });
    stripeCustomerId = customer.id;

    await db.update(users).set({ stripeCustomerId }).where(eq(users.id, userId));
  }

  // Get seller's Stripe Connect ID
  const seller = await db.select().from(users).where(eq(users.id, s.sellerId)).limit(1);
  const sellerConnectId = seller[0]?.stripeConnectId;

  // Calculate revenue split
  const tier = s.tier as keyof typeof REVENUE_SPLITS;
  const split = REVENUE_SPLITS[tier] || REVENUE_SPLITS.community;
  const platformFeeCents = Math.round(s.priceCents * (split.platform / 100));
  const stripeFeeCents = Math.round(s.priceCents * (split.stripe / 100));
  const sellerPayoutCents = s.priceCents - platformFeeCents - stripeFeeCents;

  // Create checkout session
  const session = await stripe.checkout.sessions.create({
    customer: stripeCustomerId,
    mode: 'payment',
    line_items: [
      {
        price_data: {
          currency: s.currency?.toLowerCase() || 'usd',
          product_data: {
            name: s.name,
            description: s.description,
          },
          unit_amount: s.priceCents,
        },
        quantity: 1,
      },
    ],
    payment_intent_data: sellerConnectId
      ? {
          application_fee_amount: platformFeeCents,
          transfer_data: {
            destination: sellerConnectId,
          },
        }
      : undefined,
    metadata: {
      skill_id: s.id,
      user_id: userId,
      seller_payout_cents: sellerPayoutCents.toString(),
      platform_fee_cents: platformFeeCents.toString(),
      stripe_fee_cents: stripeFeeCents.toString(),
    },
    success_url: `${FRONTEND_URL}/purchase/success?session_id={CHECKOUT_SESSION_ID}`,
    cancel_url: `${FRONTEND_URL}/marketplace/${slug}?canceled=true`,
  });

  return c.json({
    checkout_url: session.url,
    session_id: session.id,
  });
});

// List user's purchases
purchasesRoutes.get('/', async (c) => {
  const token = getCookie(c, 'auth_token');

  if (!token) {
    return c.json({ error: 'Authentication required' }, 401);
  }

  let userId: string;
  try {
    const payload = await verify(token, JWT_SECRET);
    userId = payload.sub;
  } catch {
    return c.json({ error: 'Invalid token' }, 401);
  }

  const results = await db
    .select({
      id: purchases.id,
      licenseKey: purchases.licenseKey,
      licenseStatus: purchases.licenseStatus,
      amountCents: purchases.amountCents,
      purchasedAt: purchases.purchasedAt,
      skill: {
        id: skills.id,
        name: skills.name,
        slug: skills.slug,
        version: skills.version,
        description: skills.description,
      },
    })
    .from(purchases)
    .leftJoin(skills, eq(purchases.skillId, skills.id))
    .where(eq(purchases.userId, userId));

  return c.json({ purchases: results });
});

// Get purchase details
purchasesRoutes.get('/:id', async (c) => {
  const id = c.req.param('id');
  const token = getCookie(c, 'auth_token');

  if (!token) {
    return c.json({ error: 'Authentication required' }, 401);
  }

  let userId: string;
  try {
    const payload = await verify(token, JWT_SECRET);
    userId = payload.sub;
  } catch {
    return c.json({ error: 'Invalid token' }, 401);
  }

  const purchase = await db
    .select({
      id: purchases.id,
      licenseKey: purchases.licenseKey,
      licenseStatus: purchases.licenseStatus,
      machineFingerprints: purchases.machineFingerprints,
      amountCents: purchases.amountCents,
      purchasedAt: purchases.purchasedAt,
      skill: {
        id: skills.id,
        name: skills.name,
        slug: skills.slug,
        version: skills.version,
      },
    })
    .from(purchases)
    .leftJoin(skills, eq(purchases.skillId, skills.id))
    .where(and(eq(purchases.id, id), eq(purchases.userId, userId)))
    .limit(1);

  if (purchase.length === 0) {
    return c.json({ error: 'Purchase not found' }, 404);
  }

  return c.json(purchase[0]);
});

// Request refund (within 7 days)
purchasesRoutes.post('/:id/refund', async (c) => {
  const id = c.req.param('id');
  const token = getCookie(c, 'auth_token');

  if (!token) {
    return c.json({ error: 'Authentication required' }, 401);
  }

  let userId: string;
  try {
    const payload = await verify(token, JWT_SECRET);
    userId = payload.sub;
  } catch {
    return c.json({ error: 'Invalid token' }, 401);
  }

  const purchase = await db
    .select()
    .from(purchases)
    .where(and(eq(purchases.id, id), eq(purchases.userId, userId)))
    .limit(1);

  if (purchase.length === 0) {
    return c.json({ error: 'Purchase not found' }, 404);
  }

  const p = purchase[0];

  // Check refund eligibility (within 7 days)
  const purchaseDate = new Date(p.purchasedAt!);
  const daysSincePurchase = (Date.now() - purchaseDate.getTime()) / (1000 * 60 * 60 * 24);

  if (daysSincePurchase > 7) {
    return c.json({ error: 'Refund period expired (7 days)' }, 400);
  }

  if (p.licenseStatus !== 'active') {
    return c.json({ error: 'Purchase already refunded or revoked' }, 400);
  }

  // Process refund via Stripe
  if (p.stripePaymentIntentId) {
    await stripe.refunds.create({
      payment_intent: p.stripePaymentIntentId,
    });
  }

  // Update purchase status
  await db
    .update(purchases)
    .set({
      licenseStatus: 'refunded',
      refundedAt: new Date(),
    })
    .where(eq(purchases.id, id));

  return c.json({ success: true, message: 'Refund processed' });
});

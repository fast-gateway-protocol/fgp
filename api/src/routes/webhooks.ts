import { Hono } from 'hono';
import Stripe from 'stripe';
import { eq, desc, sql } from 'drizzle-orm';
import { db, purchases, skills, skillVersions, users } from '../db';

export const webhooksRoutes = new Hono();

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);

const WEBHOOK_SECRET = process.env.STRIPE_WEBHOOK_SECRET!;

// Stripe webhook handler
webhooksRoutes.post('/stripe', async (c) => {
  const signature = c.req.header('stripe-signature');
  if (!signature) {
    return c.json({ error: 'Missing signature' }, 400);
  }

  const body = await c.req.text();

  let event: Stripe.Event;
  try {
    event = stripe.webhooks.constructEvent(body, signature, WEBHOOK_SECRET);
  } catch (err) {
    console.error('Webhook signature verification failed:', err);
    return c.json({ error: 'Invalid signature' }, 400);
  }

  switch (event.type) {
    case 'checkout.session.completed': {
      const session = event.data.object as Stripe.Checkout.Session;
      await handleCheckoutComplete(session);
      break;
    }

    case 'payment_intent.payment_failed': {
      const paymentIntent = event.data.object as Stripe.PaymentIntent;
      console.log('Payment failed:', paymentIntent.id);
      break;
    }

    case 'charge.refunded': {
      const charge = event.data.object as Stripe.Charge;
      await handleRefund(charge);
      break;
    }

    default:
      console.log(`Unhandled event type: ${event.type}`);
  }

  return c.json({ received: true });
});

async function handleCheckoutComplete(session: Stripe.Checkout.Session) {
  const metadata = session.metadata;
  if (!metadata?.skill_id || !metadata?.user_id) {
    console.error('Missing metadata in checkout session');
    return;
  }

  const skillId = metadata.skill_id;
  const userId = metadata.user_id;
  const sellerPayoutCents = parseInt(metadata.seller_payout_cents || '0');
  const platformFeeCents = parseInt(metadata.platform_fee_cents || '0');
  const stripeFeeCents = parseInt(metadata.stripe_fee_cents || '0');

  // Get latest skill version
  const latestVersion = await db
    .select()
    .from(skillVersions)
    .where(eq(skillVersions.skillId, skillId))
    .orderBy(desc(skillVersions.createdAt))
    .limit(1);

  if (latestVersion.length === 0) {
    console.error('No skill version found for:', skillId);
    return;
  }

  // Create purchase record
  await db.insert(purchases).values({
    userId,
    skillId,
    skillVersionId: latestVersion[0].id,
    stripePaymentIntentId: session.payment_intent as string,
    stripeCheckoutSessionId: session.id,
    amountCents: session.amount_total || 0,
    sellerPayoutCents,
    platformFeeCents,
    stripeFeeCents,
  });

  // Increment download count
  await db
    .update(skills)
    .set({
      downloadCount: sql`${skills.downloadCount} + 1`,
    })
    .where(eq(skills.id, skillId));

  // TODO: Send confirmation email
  console.log(`Purchase completed: user=${userId}, skill=${skillId}`);
}

async function handleRefund(charge: Stripe.Charge) {
  const paymentIntentId = charge.payment_intent as string;
  if (!paymentIntentId) return;

  // Update purchase status
  await db
    .update(purchases)
    .set({
      licenseStatus: 'refunded',
      refundedAt: new Date(),
    })
    .where(eq(purchases.stripePaymentIntentId, paymentIntentId));

  console.log(`Refund processed for payment intent: ${paymentIntentId}`);
}

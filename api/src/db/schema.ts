import {
  pgTable,
  uuid,
  varchar,
  text,
  integer,
  decimal,
  boolean,
  timestamp,
  jsonb,
  inet,
  unique,
  index,
  check,
} from 'drizzle-orm/pg-core';
import { sql } from 'drizzle-orm';

// Users table (buyers and sellers)
export const users = pgTable(
  'users',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    email: varchar('email', { length: 255 }).unique().notNull(),
    name: varchar('name', { length: 255 }),
    githubId: varchar('github_id', { length: 100 }).unique(),
    githubUsername: varchar('github_username', { length: 100 }),
    avatarUrl: varchar('avatar_url', { length: 500 }),
    stripeCustomerId: varchar('stripe_customer_id', { length: 100 }),
    stripeConnectId: varchar('stripe_connect_id', { length: 100 }),
    role: varchar('role', { length: 20 }).default('buyer'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
    updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow(),
  },
  (table) => [index('idx_users_github_id').on(table.githubId)]
);

// Skills table
export const skills = pgTable(
  'skills',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    name: varchar('name', { length: 64 }).unique().notNull(),
    slug: varchar('slug', { length: 64 }).unique().notNull(),
    sellerId: uuid('seller_id')
      .references(() => users.id)
      .notNull(),

    // Pricing
    priceCents: integer('price_cents').notNull().default(0),
    currency: varchar('currency', { length: 3 }).default('USD'),
    tier: varchar('tier', { length: 20 }).default('free'),
    stripeProductId: varchar('stripe_product_id', { length: 100 }),
    stripePriceId: varchar('stripe_price_id', { length: 100 }),

    // Metadata
    version: varchar('version', { length: 20 }).notNull(),
    description: text('description').notNull(),
    readmeHtml: text('readme_html'),
    manifestJson: jsonb('manifest_json').notNull(),

    // Content
    previewRepo: varchar('preview_repo', { length: 500 }),
    demoVideoUrl: varchar('demo_video_url', { length: 500 }),
    screenshots: jsonb('screenshots').default([]),

    // Stats
    downloadCount: integer('download_count').default(0),
    ratingAvg: decimal('rating_avg', { precision: 3, scale: 2 }).default('0'),
    ratingCount: integer('rating_count').default(0),

    // Status
    status: varchar('status', { length: 20 }).default('pending'),
    verifiedAt: timestamp('verified_at', { withTimezone: true }),
    verifiedBy: uuid('verified_by').references(() => users.id),

    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
    updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow(),
  },
  (table) => [
    index('idx_skills_seller').on(table.sellerId),
    index('idx_skills_tier').on(table.tier),
    index('idx_skills_status').on(table.status),
    index('idx_skills_slug').on(table.slug),
  ]
);

// Skill versions table
export const skillVersions = pgTable(
  'skill_versions',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    skillId: uuid('skill_id')
      .references(() => skills.id, { onDelete: 'cascade' })
      .notNull(),
    version: varchar('version', { length: 20 }).notNull(),
    manifestJson: jsonb('manifest_json').notNull(),
    archiveUrl: varchar('archive_url', { length: 500 }),
    archiveChecksum: varchar('archive_checksum', { length: 64 }),
    archiveSizeBytes: integer('archive_size_bytes'),
    changelog: text('changelog'),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
  },
  (table) => [unique('skill_versions_skill_version').on(table.skillId, table.version)]
);

// Purchases / Licenses table
export const purchases = pgTable(
  'purchases',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    userId: uuid('user_id')
      .references(() => users.id)
      .notNull(),
    skillId: uuid('skill_id')
      .references(() => skills.id)
      .notNull(),
    skillVersionId: uuid('skill_version_id')
      .references(() => skillVersions.id)
      .notNull(),

    // Stripe
    stripePaymentIntentId: varchar('stripe_payment_intent_id', { length: 100 }),
    stripeCheckoutSessionId: varchar('stripe_checkout_session_id', { length: 100 }),

    // License
    licenseKey: uuid('license_key').defaultRandom().unique(),
    licenseStatus: varchar('license_status', { length: 20 }).default('active'),
    machineFingerprints: jsonb('machine_fingerprints').default([]),

    // Financial
    amountCents: integer('amount_cents').notNull(),
    sellerPayoutCents: integer('seller_payout_cents').notNull(),
    platformFeeCents: integer('platform_fee_cents').notNull(),
    stripeFeeCents: integer('stripe_fee_cents').notNull(),

    purchasedAt: timestamp('purchased_at', { withTimezone: true }).defaultNow(),
    refundedAt: timestamp('refunded_at', { withTimezone: true }),
  },
  (table) => [
    unique('purchases_user_skill').on(table.userId, table.skillId),
    index('idx_purchases_user').on(table.userId),
    index('idx_purchases_skill').on(table.skillId),
    index('idx_purchases_license').on(table.licenseKey),
  ]
);

// Reviews table
export const reviews = pgTable(
  'reviews',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    skillId: uuid('skill_id')
      .references(() => skills.id, { onDelete: 'cascade' })
      .notNull(),
    userId: uuid('user_id')
      .references(() => users.id)
      .notNull(),
    purchaseId: uuid('purchase_id')
      .references(() => purchases.id)
      .notNull(),
    rating: integer('rating').notNull(),
    title: varchar('title', { length: 255 }),
    body: text('body'),
    helpfulCount: integer('helpful_count').default(0),
    createdAt: timestamp('created_at', { withTimezone: true }).defaultNow(),
    updatedAt: timestamp('updated_at', { withTimezone: true }).defaultNow(),
  },
  (table) => [
    unique('reviews_skill_user').on(table.skillId, table.userId),
    index('idx_reviews_skill').on(table.skillId),
  ]
);

// License validations audit table
export const licenseValidations = pgTable(
  'license_validations',
  {
    id: uuid('id').primaryKey().defaultRandom(),
    purchaseId: uuid('purchase_id')
      .references(() => purchases.id)
      .notNull(),
    machineFingerprint: varchar('machine_fingerprint', { length: 64 }).notNull(),
    ipAddress: varchar('ip_address', { length: 45 }),
    userAgent: text('user_agent'),
    result: varchar('result', { length: 20 }).notNull(),
    validatedAt: timestamp('validated_at', { withTimezone: true }).defaultNow(),
  },
  (table) => [index('idx_license_validations_purchase').on(table.purchaseId)]
);

// Type exports
export type User = typeof users.$inferSelect;
export type NewUser = typeof users.$inferInsert;
export type Skill = typeof skills.$inferSelect;
export type NewSkill = typeof skills.$inferInsert;
export type SkillVersion = typeof skillVersions.$inferSelect;
export type NewSkillVersion = typeof skillVersions.$inferInsert;
export type Purchase = typeof purchases.$inferSelect;
export type NewPurchase = typeof purchases.$inferInsert;
export type Review = typeof reviews.$inferSelect;
export type NewReview = typeof reviews.$inferInsert;

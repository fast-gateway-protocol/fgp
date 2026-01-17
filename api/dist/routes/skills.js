import { Hono } from 'hono';
import { eq, desc, sql, ilike, or, and } from 'drizzle-orm';
import { db, skills, users, skillVersions, reviews } from '../db';
export const skillsRoutes = new Hono();
// List skills (public)
skillsRoutes.get('/', async (c) => {
    const tier = c.req.query('tier');
    const status = c.req.query('status') || 'active';
    const search = c.req.query('search');
    const limit = parseInt(c.req.query('limit') || '20');
    const offset = parseInt(c.req.query('offset') || '0');
    let conditions = [eq(skills.status, status)];
    if (tier && tier !== 'all') {
        conditions.push(eq(skills.tier, tier));
    }
    if (search) {
        conditions.push(or(ilike(skills.name, `%${search}%`), ilike(skills.description, `%${search}%`)));
    }
    const results = await db
        .select({
        id: skills.id,
        name: skills.name,
        slug: skills.slug,
        version: skills.version,
        description: skills.description,
        priceCents: skills.priceCents,
        currency: skills.currency,
        tier: skills.tier,
        downloadCount: skills.downloadCount,
        ratingAvg: skills.ratingAvg,
        ratingCount: skills.ratingCount,
        previewRepo: skills.previewRepo,
        demoVideoUrl: skills.demoVideoUrl,
        screenshots: skills.screenshots,
        createdAt: skills.createdAt,
        seller: {
            id: users.id,
            name: users.name,
            githubUsername: users.githubUsername,
            avatarUrl: users.avatarUrl,
        },
    })
        .from(skills)
        .leftJoin(users, eq(skills.sellerId, users.id))
        .where(and(...conditions))
        .orderBy(desc(skills.downloadCount))
        .limit(limit)
        .offset(offset);
    const countResult = await db
        .select({ count: sql `count(*)` })
        .from(skills)
        .where(and(...conditions));
    return c.json({
        skills: results,
        total: countResult[0]?.count || 0,
        limit,
        offset,
    });
});
// Get skill by slug (public)
skillsRoutes.get('/:slug', async (c) => {
    const slug = c.req.param('slug');
    const result = await db
        .select({
        id: skills.id,
        name: skills.name,
        slug: skills.slug,
        version: skills.version,
        description: skills.description,
        readmeHtml: skills.readmeHtml,
        manifestJson: skills.manifestJson,
        priceCents: skills.priceCents,
        currency: skills.currency,
        tier: skills.tier,
        stripeProductId: skills.stripeProductId,
        stripePriceId: skills.stripePriceId,
        downloadCount: skills.downloadCount,
        ratingAvg: skills.ratingAvg,
        ratingCount: skills.ratingCount,
        previewRepo: skills.previewRepo,
        demoVideoUrl: skills.demoVideoUrl,
        screenshots: skills.screenshots,
        status: skills.status,
        verifiedAt: skills.verifiedAt,
        createdAt: skills.createdAt,
        updatedAt: skills.updatedAt,
        seller: {
            id: users.id,
            name: users.name,
            githubUsername: users.githubUsername,
            avatarUrl: users.avatarUrl,
        },
    })
        .from(skills)
        .leftJoin(users, eq(skills.sellerId, users.id))
        .where(eq(skills.slug, slug))
        .limit(1);
    if (result.length === 0) {
        return c.json({ error: 'Skill not found' }, 404);
    }
    return c.json(result[0]);
});
// Get skill versions
skillsRoutes.get('/:slug/versions', async (c) => {
    const slug = c.req.param('slug');
    const skill = await db.select({ id: skills.id }).from(skills).where(eq(skills.slug, slug)).limit(1);
    if (skill.length === 0) {
        return c.json({ error: 'Skill not found' }, 404);
    }
    const versions = await db
        .select({
        id: skillVersions.id,
        version: skillVersions.version,
        changelog: skillVersions.changelog,
        archiveSizeBytes: skillVersions.archiveSizeBytes,
        createdAt: skillVersions.createdAt,
    })
        .from(skillVersions)
        .where(eq(skillVersions.skillId, skill[0].id))
        .orderBy(desc(skillVersions.createdAt));
    return c.json({ versions });
});
// Get skill reviews
skillsRoutes.get('/:slug/reviews', async (c) => {
    const slug = c.req.param('slug');
    const limit = parseInt(c.req.query('limit') || '10');
    const offset = parseInt(c.req.query('offset') || '0');
    const skill = await db.select({ id: skills.id }).from(skills).where(eq(skills.slug, slug)).limit(1);
    if (skill.length === 0) {
        return c.json({ error: 'Skill not found' }, 404);
    }
    const results = await db
        .select({
        id: reviews.id,
        rating: reviews.rating,
        title: reviews.title,
        body: reviews.body,
        helpfulCount: reviews.helpfulCount,
        createdAt: reviews.createdAt,
        user: {
            id: users.id,
            name: users.name,
            githubUsername: users.githubUsername,
            avatarUrl: users.avatarUrl,
        },
    })
        .from(reviews)
        .leftJoin(users, eq(reviews.userId, users.id))
        .where(eq(reviews.skillId, skill[0].id))
        .orderBy(desc(reviews.createdAt))
        .limit(limit)
        .offset(offset);
    return c.json({ reviews: results });
});
// Categories (static for now)
skillsRoutes.get('/meta/categories', (c) => {
    return c.json({
        categories: [
            { id: 'browser-automation', name: 'Browser Automation' },
            { id: 'email', name: 'Email' },
            { id: 'calendar', name: 'Calendar' },
            { id: 'development', name: 'Development' },
            { id: 'database', name: 'Database' },
            { id: 'deployment', name: 'Deployment' },
            { id: 'messaging', name: 'Messaging' },
            { id: 'productivity', name: 'Productivity' },
            { id: 'research', name: 'Research' },
            { id: 'data', name: 'Data Processing' },
        ],
    });
});
// Tiers info
skillsRoutes.get('/meta/tiers', (c) => {
    return c.json({
        tiers: [
            { id: 'free', name: 'Free', priceRange: '$0', platformFee: 0, sellerShare: 100 },
            { id: 'community', name: 'Community', priceRange: '$5-29', platformFee: 30, sellerShare: 65 },
            { id: 'verified', name: 'Verified', priceRange: '$30-99', platformFee: 25, sellerShare: 70 },
            { id: 'pro', name: 'Pro', priceRange: '$100-499', platformFee: 20, sellerShare: 75 },
        ],
    });
});
//# sourceMappingURL=skills.js.map
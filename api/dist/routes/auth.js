import { Hono } from 'hono';
import { getCookie, setCookie, deleteCookie } from 'hono/cookie';
import { eq } from 'drizzle-orm';
import { db, users } from '../db';
import { sign, verify } from '../lib/jwt';
export const authRoutes = new Hono();
const GITHUB_CLIENT_ID = process.env.GITHUB_CLIENT_ID;
const GITHUB_CLIENT_SECRET = process.env.GITHUB_CLIENT_SECRET;
const JWT_SECRET = process.env.JWT_SECRET;
const FRONTEND_URL = process.env.FRONTEND_URL || 'https://fgp.dev';
// Initiate GitHub OAuth
authRoutes.get('/github', (c) => {
    const redirectUri = `${c.req.url.split('/api')[0]}/api/v1/auth/github/callback`;
    const state = crypto.randomUUID();
    // Store state in cookie for CSRF protection
    setCookie(c, 'oauth_state', state, {
        httpOnly: true,
        secure: true,
        sameSite: 'Lax',
        maxAge: 60 * 10, // 10 minutes
    });
    const params = new URLSearchParams({
        client_id: GITHUB_CLIENT_ID,
        redirect_uri: redirectUri,
        scope: 'read:user user:email',
        state,
    });
    return c.redirect(`https://github.com/login/oauth/authorize?${params}`);
});
// GitHub OAuth callback
authRoutes.get('/github/callback', async (c) => {
    const code = c.req.query('code');
    const state = c.req.query('state');
    const storedState = getCookie(c, 'oauth_state');
    // Clear state cookie
    deleteCookie(c, 'oauth_state');
    if (!code || !state || state !== storedState) {
        return c.redirect(`${FRONTEND_URL}/login?error=invalid_state`);
    }
    try {
        // Exchange code for access token
        const tokenResponse = await fetch('https://github.com/login/oauth/access_token', {
            method: 'POST',
            headers: {
                Accept: 'application/json',
                'Content-Type': 'application/json',
            },
            body: JSON.stringify({
                client_id: GITHUB_CLIENT_ID,
                client_secret: GITHUB_CLIENT_SECRET,
                code,
            }),
        });
        const tokenData = (await tokenResponse.json());
        if (tokenData.error || !tokenData.access_token) {
            return c.redirect(`${FRONTEND_URL}/login?error=token_exchange_failed`);
        }
        // Get user info from GitHub
        const userResponse = await fetch('https://api.github.com/user', {
            headers: {
                Authorization: `Bearer ${tokenData.access_token}`,
                Accept: 'application/vnd.github.v3+json',
            },
        });
        const githubUser = (await userResponse.json());
        // Get user email (may be private)
        let email = githubUser.email;
        if (!email) {
            const emailsResponse = await fetch('https://api.github.com/user/emails', {
                headers: {
                    Authorization: `Bearer ${tokenData.access_token}`,
                    Accept: 'application/vnd.github.v3+json',
                },
            });
            const emails = (await emailsResponse.json());
            const primaryEmail = emails.find((e) => e.primary);
            email = primaryEmail?.email || emails[0]?.email || null;
        }
        if (!email) {
            return c.redirect(`${FRONTEND_URL}/login?error=no_email`);
        }
        // Find or create user
        let user = await db
            .select()
            .from(users)
            .where(eq(users.githubId, String(githubUser.id)))
            .limit(1);
        if (user.length === 0) {
            // Create new user
            const newUser = await db
                .insert(users)
                .values({
                email,
                name: githubUser.name || githubUser.login,
                githubId: String(githubUser.id),
                githubUsername: githubUser.login,
                avatarUrl: githubUser.avatar_url,
            })
                .returning();
            user = newUser;
        }
        else {
            // Update existing user
            await db
                .update(users)
                .set({
                email,
                name: githubUser.name || githubUser.login,
                githubUsername: githubUser.login,
                avatarUrl: githubUser.avatar_url,
                updatedAt: new Date(),
            })
                .where(eq(users.githubId, String(githubUser.id)));
        }
        // Create JWT token
        const token = await sign({
            sub: user[0].id,
            email: user[0].email,
            role: user[0].role,
        }, JWT_SECRET);
        // Set auth cookie
        setCookie(c, 'auth_token', token, {
            httpOnly: true,
            secure: true,
            sameSite: 'Lax',
            maxAge: 60 * 60 * 24 * 7, // 7 days
            path: '/',
        });
        return c.redirect(`${FRONTEND_URL}/dashboard`);
    }
    catch (error) {
        console.error('OAuth error:', error);
        return c.redirect(`${FRONTEND_URL}/login?error=oauth_failed`);
    }
});
// Get current user
authRoutes.get('/me', async (c) => {
    const token = getCookie(c, 'auth_token');
    if (!token) {
        return c.json({ user: null });
    }
    try {
        const payload = await verify(token, JWT_SECRET);
        const user = await db.select().from(users).where(eq(users.id, payload.sub)).limit(1);
        if (user.length === 0) {
            return c.json({ user: null });
        }
        return c.json({
            user: {
                id: user[0].id,
                email: user[0].email,
                name: user[0].name,
                githubUsername: user[0].githubUsername,
                avatarUrl: user[0].avatarUrl,
                role: user[0].role,
                stripeConnectId: user[0].stripeConnectId,
                createdAt: user[0].createdAt,
            },
        });
    }
    catch (error) {
        return c.json({ user: null });
    }
});
// Logout
authRoutes.post('/logout', (c) => {
    deleteCookie(c, 'auth_token', { path: '/' });
    return c.json({ success: true });
});
// Upgrade to seller (initiates Stripe Connect)
authRoutes.post('/become-seller', async (c) => {
    const token = getCookie(c, 'auth_token');
    if (!token) {
        return c.json({ error: 'Unauthorized' }, 401);
    }
    try {
        const payload = await verify(token, JWT_SECRET);
        const user = await db.select().from(users).where(eq(users.id, payload.sub)).limit(1);
        if (user.length === 0) {
            return c.json({ error: 'User not found' }, 404);
        }
        if (user[0].role === 'seller' || user[0].role === 'admin') {
            return c.json({ error: 'Already a seller' }, 400);
        }
        // Update role to seller
        await db.update(users).set({ role: 'seller', updatedAt: new Date() }).where(eq(users.id, user[0].id));
        return c.json({ success: true, message: 'You are now a seller' });
    }
    catch (error) {
        return c.json({ error: 'Invalid token' }, 401);
    }
});
//# sourceMappingURL=auth.js.map
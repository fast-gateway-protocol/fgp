import { Hono } from 'hono';
import { cors } from 'hono/cors';
import { logger } from 'hono/logger';
import { skillsRoutes } from './routes/skills';
import { authRoutes } from './routes/auth';
import { licensesRoutes } from './routes/licenses';
import { purchasesRoutes } from './routes/purchases';
import { webhooksRoutes } from './routes/webhooks';

const app = new Hono();

// Middleware
app.use('*', logger());
app.use(
  '*',
  cors({
    origin: ['https://fgp.dev', 'https://getfgp.com', 'http://localhost:5173'],
    allowMethods: ['GET', 'POST', 'PUT', 'DELETE', 'OPTIONS'],
    allowHeaders: ['Content-Type', 'Authorization'],
    credentials: true,
  })
);

// Health check
app.get('/health', (c) => c.json({ status: 'ok', timestamp: new Date().toISOString() }));

// API routes
app.route('/api/v1/auth', authRoutes);
app.route('/api/v1/skills', skillsRoutes);
app.route('/api/v1/licenses', licensesRoutes);
app.route('/api/v1/purchases', purchasesRoutes);
app.route('/webhooks', webhooksRoutes);

// 404 handler
app.notFound((c) => c.json({ error: 'Not found' }, 404));

// Error handler
app.onError((err, c) => {
  console.error('Error:', err);
  return c.json({ error: 'Internal server error' }, 500);
});

// Export for Vercel Edge Functions
export default app;

// Local development server
import { serve } from '@hono/node-server';

if (process.env.NODE_ENV !== 'production') {
  const port = parseInt(process.env.PORT || '3001');
  serve({ fetch: app.fetch, port }, (info) => {
    console.log(`FGP Marketplace API running on http://localhost:${info.port}`);
  });
}

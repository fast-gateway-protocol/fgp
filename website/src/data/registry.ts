export interface PackageMethod {
  name: string;
  description: string;
  params?: {
    name: string;
    type: string;
    required: boolean;
    default?: unknown;
  }[];
}

export interface Package {
  name: string;
  version: string;
  description: string;
  repository: string;
  license: string;
  platforms: string[];
  categories: string[];
  featured: boolean;
  verified: boolean;
  skills: string[];
  auth: {
    type: string;
    provider?: string;
    scopes?: string[];
    setup?: string;
  };
  methods: PackageMethod[];
  benchmark?: {
    avg_latency_ms: number;
    vs_mcp_speedup: string;
  };
  added_at: string;
  updated_at: string;
  // Marketplace fields
  price_cents?: number;
  currency?: string;
  tier?: 'free' | 'community' | 'verified' | 'pro';
  seller?: {
    name: string;
    avatar?: string;
  };
  download_count?: number;
  rating?: {
    average: number;
    count: number;
  };
}

export const packages: Package[] = [
  {
    name: 'browser',
    version: '0.1.0',
    description: 'Browser automation via Chrome DevTools Protocol - 3-12x faster than MCP (warm), eliminates cold start',
    repository: 'https://github.com/fast-gateway-protocol/browser',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['browser', 'automation'],
    featured: true,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf', 'continue'],
    auth: {
      type: 'stateful',
      setup: 'Auth state saved via browser.state.save/load',
    },
    methods: [
      { name: 'browser.open', description: 'Navigate to a URL' },
      { name: 'browser.snapshot', description: 'Get ARIA accessibility tree with @eN refs' },
      { name: 'browser.screenshot', description: 'Capture screenshot (base64 or file)' },
      { name: 'browser.click', description: 'Click element by @eN ref or CSS selector' },
      { name: 'browser.fill', description: 'Fill input field with value' },
      { name: 'browser.press', description: 'Press a keyboard key' },
      { name: 'browser.select', description: 'Select an option from a dropdown' },
      { name: 'browser.check', description: 'Set checkbox/radio state' },
      { name: 'browser.hover', description: 'Hover over an element' },
      { name: 'browser.scroll', description: 'Scroll to element or by amount' },
      { name: 'browser.press_combo', description: 'Press key with modifiers (Ctrl, Shift, Alt, Meta)' },
      { name: 'browser.upload', description: 'Upload a file to a file input element' },
      { name: 'browser.state.save', description: 'Save auth state (cookies + localStorage)' },
      { name: 'browser.state.load', description: 'Load saved auth state' },
      { name: 'browser.state.list', description: 'List saved auth states' },
      { name: 'session.new', description: 'Create a new isolated browser session' },
      { name: 'session.list', description: 'List all active sessions' },
      { name: 'session.close', description: 'Close and dispose a session' },
    ],
    benchmark: {
      avg_latency_ms: 5,
      vs_mcp_speedup: '3-12x',
    },
    added_at: '2025-12-01',
    updated_at: '2026-01-14',
  },
  {
    name: 'gmail',
    version: '0.1.0',
    description: 'Fast Gmail integration via FGP daemon - read, search, and send emails',
    repository: 'https://github.com/fast-gateway-protocol/gmail',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['productivity', 'email'],
    featured: true,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf'],
    auth: {
      type: 'oauth2',
      provider: 'google',
      scopes: [
        'https://www.googleapis.com/auth/gmail.readonly',
        'https://www.googleapis.com/auth/gmail.send',
        'https://www.googleapis.com/auth/gmail.modify',
      ],
    },
    methods: [
      {
        name: 'gmail.inbox',
        description: 'List recent inbox emails',
        params: [{ name: 'limit', type: 'integer', required: false, default: 10 }],
      },
      { name: 'gmail.unread', description: 'Get unread email count and summaries' },
      {
        name: 'gmail.search',
        description: 'Search emails by query',
        params: [
          { name: 'query', type: 'string', required: true },
          { name: 'limit', type: 'integer', required: false, default: 10 },
        ],
      },
      {
        name: 'gmail.send',
        description: 'Send an email',
        params: [
          { name: 'to', type: 'string', required: true },
          { name: 'subject', type: 'string', required: true },
          { name: 'body', type: 'string', required: true },
        ],
      },
      {
        name: 'gmail.thread',
        description: 'Get email thread by ID',
        params: [{ name: 'thread_id', type: 'string', required: true }],
      },
    ],
    benchmark: {
      avg_latency_ms: 35,
      vs_mcp_speedup: '~10x cold',
    },
    added_at: '2025-11-15',
    updated_at: '2026-01-10',
  },
  {
    name: 'calendar',
    version: '0.1.0',
    description: 'Fast Google Calendar integration - events, scheduling, and free slots',
    repository: 'https://github.com/fast-gateway-protocol/calendar',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['productivity', 'calendar'],
    featured: true,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf'],
    auth: {
      type: 'oauth2',
      provider: 'google',
      scopes: ['https://www.googleapis.com/auth/calendar'],
    },
    methods: [
      { name: 'calendar.today', description: "Get today's calendar events" },
      {
        name: 'calendar.upcoming',
        description: 'Get upcoming events',
        params: [
          { name: 'days', type: 'integer', required: false, default: 7 },
          { name: 'limit', type: 'integer', required: false, default: 20 },
        ],
      },
      {
        name: 'calendar.search',
        description: 'Search events by query',
        params: [
          { name: 'query', type: 'string', required: true },
          { name: 'days', type: 'integer', required: false, default: 30 },
        ],
      },
      {
        name: 'calendar.create',
        description: 'Create a new event',
        params: [
          { name: 'summary', type: 'string', required: true },
          { name: 'start', type: 'string', required: true },
          { name: 'end', type: 'string', required: true },
          { name: 'description', type: 'string', required: false },
        ],
      },
      {
        name: 'calendar.free_slots',
        description: 'Find available time slots',
        params: [
          { name: 'duration_minutes', type: 'integer', required: true },
          { name: 'days', type: 'integer', required: false, default: 7 },
        ],
      },
    ],
    benchmark: {
      avg_latency_ms: 233,
      vs_mcp_speedup: '10x',
    },
    added_at: '2025-11-20',
    updated_at: '2026-01-08',
  },
  {
    name: 'github',
    version: '0.1.0',
    description: 'GitHub operations via gh CLI - repos, issues, PRs, and notifications',
    repository: 'https://github.com/fast-gateway-protocol/github',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['devtools', 'productivity'],
    featured: true,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf'],
    auth: {
      type: 'cli',
      provider: 'gh',
      setup: 'gh auth login',
    },
    methods: [
      { name: 'github.repos', description: 'List your repositories' },
      { name: 'github.issues', description: 'List issues for a repository' },
      { name: 'github.notifications', description: 'Get unread notifications' },
      { name: 'github.pr_status', description: 'Check PR status for current branch' },
      { name: 'github.user', description: 'Get current authenticated user' },
    ],
    benchmark: {
      avg_latency_ms: 474,
      vs_mcp_speedup: '4x',
    },
    added_at: '2025-10-15',
    updated_at: '2026-01-05',
  },
  {
    name: 'fly',
    version: '0.1.0',
    description: 'Fly.io operations via GraphQL API - apps, machines, and deployments',
    repository: 'https://github.com/fast-gateway-protocol/fly',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['cloud', 'devtools'],
    featured: false,
    verified: true,
    skills: ['claude-code'],
    auth: {
      type: 'bearer_token',
      provider: 'fly.io',
      setup: 'Set FLY_API_TOKEN environment variable',
    },
    methods: [
      {
        name: 'fly.apps',
        description: 'List all Fly.io apps',
        params: [{ name: 'limit', type: 'integer', required: false, default: 25 }],
      },
      {
        name: 'fly.status',
        description: 'Get status for a specific app',
        params: [{ name: 'app', type: 'string', required: true }],
      },
      {
        name: 'fly.machines',
        description: 'List machines for an app',
        params: [{ name: 'app', type: 'string', required: true }],
      },
      { name: 'fly.user', description: 'Get current user info' },
    ],
    added_at: '2026-01-02',
    updated_at: '2026-01-12',
  },
  {
    name: 'neon',
    version: '0.1.0',
    description: 'Neon serverless Postgres operations - projects, branches, and SQL queries',
    repository: 'https://github.com/fast-gateway-protocol/neon',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['cloud', 'database'],
    featured: false,
    verified: true,
    skills: ['claude-code'],
    auth: {
      type: 'bearer_token',
      provider: 'neon.tech',
      setup: 'Set NEON_API_KEY and NEON_ORG_ID environment variables',
    },
    methods: [
      {
        name: 'neon.projects',
        description: 'List all Neon projects',
        params: [{ name: 'limit', type: 'integer', required: false, default: 10 }],
      },
      {
        name: 'neon.project',
        description: 'Get a specific project',
        params: [{ name: 'project_id', type: 'string', required: true }],
      },
      {
        name: 'neon.branches',
        description: 'List branches for a project',
        params: [{ name: 'project_id', type: 'string', required: true }],
      },
      {
        name: 'neon.databases',
        description: 'List databases for a branch',
        params: [
          { name: 'project_id', type: 'string', required: true },
          { name: 'branch_id', type: 'string', required: true },
        ],
      },
      {
        name: 'neon.tables',
        description: 'List tables in a database',
        params: [
          { name: 'project_id', type: 'string', required: true },
          { name: 'branch_id', type: 'string', required: true },
          { name: 'database', type: 'string', required: false, default: 'neondb' },
        ],
      },
      {
        name: 'neon.schema',
        description: 'Get table schema',
        params: [
          { name: 'project_id', type: 'string', required: true },
          { name: 'branch_id', type: 'string', required: true },
          { name: 'table', type: 'string', required: true },
        ],
      },
      {
        name: 'neon.sql',
        description: 'Run a SQL query',
        params: [
          { name: 'project_id', type: 'string', required: true },
          { name: 'branch_id', type: 'string', required: true },
          { name: 'query', type: 'string', required: true },
        ],
      },
      { name: 'neon.user', description: 'Get current user info' },
    ],
    added_at: '2026-01-05',
    updated_at: '2026-01-13',
  },
  {
    name: 'vercel',
    version: '0.1.0',
    description: 'Vercel deployment operations - projects, deployments, and logs',
    repository: 'https://github.com/fast-gateway-protocol/vercel',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['cloud', 'devtools'],
    featured: false,
    verified: true,
    skills: ['claude-code'],
    auth: {
      type: 'bearer_token',
      provider: 'vercel.com',
      setup: 'Set VERCEL_TOKEN environment variable',
    },
    methods: [
      {
        name: 'vercel.projects',
        description: 'List all Vercel projects',
        params: [{ name: 'limit', type: 'integer', required: false, default: 20 }],
      },
      {
        name: 'vercel.project',
        description: 'Get a specific project by ID or name',
        params: [{ name: 'project_id', type: 'string', required: true }],
      },
      {
        name: 'vercel.deployments',
        description: 'List deployments (optionally filtered by project)',
        params: [
          { name: 'project_id', type: 'string', required: false },
          { name: 'limit', type: 'integer', required: false, default: 20 },
        ],
      },
      {
        name: 'vercel.deployment',
        description: 'Get a specific deployment by ID',
        params: [{ name: 'deployment_id', type: 'string', required: true }],
      },
      {
        name: 'vercel.logs',
        description: 'Get deployment logs/events',
        params: [{ name: 'deployment_id', type: 'string', required: true }],
      },
      { name: 'vercel.user', description: 'Get current user info' },
    ],
    added_at: '2026-01-08',
    updated_at: '2026-01-14',
    tier: 'free',
    download_count: 892,
  },
  {
    name: 'travel',
    version: '1.0.0',
    description: 'Flight and hotel search via Kiwi/Xotelo APIs - token-optimized efficiency methods',
    repository: 'https://github.com/fast-gateway-protocol/travel',
    license: 'MIT',
    platforms: ['darwin', 'linux', 'windows'],
    categories: ['travel', 'productivity'],
    featured: true,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf', 'continue'],
    auth: {
      type: 'none',
      setup: 'No authentication required - uses public APIs',
    },
    methods: [
      { name: 'travel.find_location', description: 'Search airports/cities (instant, local DB)' },
      { name: 'travel.search_flights', description: 'One-way flight search' },
      { name: 'travel.search_roundtrip', description: 'Round-trip flight search' },
      { name: 'travel.search_hotels', description: 'Hotel search by city' },
      { name: 'travel.hotel_rates', description: 'Real-time hotel rates' },
      { name: 'travel.price_check', description: 'Ultra-light price check (~55 tokens, 10x more efficient)' },
      { name: 'travel.search_cheapest_day', description: 'Find cheapest day in date range (30x more efficient)' },
      { name: 'travel.search_cheapest_route', description: 'Find cheapest destination from multiple options' },
      { name: 'travel.search_flexible_dates', description: 'Search ±N days around target date' },
      { name: 'travel.search_direct_only', description: 'Non-stop flights only' },
      { name: 'travel.batch_search', description: 'Multiple searches in one call' },
      { name: 'travel.cache_stats', description: 'Cache hit/miss statistics' },
      { name: 'travel.cache_clear', description: 'Clear response cache' },
    ],
    benchmark: {
      avg_latency_ms: 5,
      vs_mcp_speedup: 'N/A (no MCP equivalent)',
    },
    added_at: '2026-01-15',
    updated_at: '2026-01-15',
    tier: 'free',
    download_count: 0,
  },
  {
    name: 'postgres',
    version: '0.1.0',
    description: 'Direct PostgreSQL operations with connection pooling - query, execute, introspect',
    repository: 'https://github.com/fast-gateway-protocol/postgres',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['database', 'devtools'],
    featured: false,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf'],
    auth: {
      type: 'connection_string',
      provider: 'postgresql',
      setup: 'Set DATABASE_URL or PGHOST/PGUSER/PGPASSWORD/PGDATABASE environment variables',
    },
    methods: [
      {
        name: 'postgres.query',
        description: 'Execute a SQL SELECT query and return results',
        params: [
          { name: 'sql', type: 'string', required: true },
          { name: 'params', type: 'array', required: false },
        ],
      },
      {
        name: 'postgres.execute',
        description: 'Execute INSERT/UPDATE/DELETE and return rows affected',
        params: [
          { name: 'sql', type: 'string', required: true },
          { name: 'params', type: 'array', required: false },
        ],
      },
      {
        name: 'postgres.transaction',
        description: 'Execute multiple statements in a transaction',
        params: [{ name: 'statements', type: 'array', required: true }],
      },
      { name: 'postgres.tables', description: 'List all tables in a schema' },
      {
        name: 'postgres.schema',
        description: 'Get table schema with columns and types',
        params: [{ name: 'table', type: 'string', required: true }],
      },
      { name: 'postgres.connections', description: 'List configured connections' },
      { name: 'postgres.version', description: 'Get PostgreSQL server version' },
    ],
    added_at: '2026-01-15',
    updated_at: '2026-01-15',
    tier: 'free',
    download_count: 0,
  },
  {
    name: 'linear',
    version: '0.1.0',
    description: 'Linear issue tracking via GraphQL - issues, projects, cycles, and comments',
    repository: 'https://github.com/fast-gateway-protocol/linear',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['devtools', 'productivity'],
    featured: true,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf'],
    auth: {
      type: 'bearer_token',
      provider: 'linear.app',
      setup: 'Set LINEAR_API_KEY environment variable (create at linear.app/settings/api)',
    },
    methods: [
      { name: 'linear.me', description: 'Get current user info' },
      { name: 'linear.teams', description: 'List all teams' },
      {
        name: 'linear.issues',
        description: 'List/search issues with filters',
        params: [
          { name: 'team', type: 'string', required: false },
          { name: 'state', type: 'string', required: false },
          { name: 'assignee', type: 'string', required: false },
          { name: 'query', type: 'string', required: false },
          { name: 'limit', type: 'integer', required: false, default: 25 },
        ],
      },
      {
        name: 'linear.issue',
        description: 'Get a single issue by ID',
        params: [{ name: 'id', type: 'string', required: true }],
      },
      {
        name: 'linear.create_issue',
        description: 'Create a new issue',
        params: [
          { name: 'title', type: 'string', required: true },
          { name: 'team_id', type: 'string', required: true },
          { name: 'description', type: 'string', required: false },
          { name: 'priority', type: 'integer', required: false },
          { name: 'assignee_id', type: 'string', required: false },
        ],
      },
      {
        name: 'linear.update_issue',
        description: 'Update an issue',
        params: [
          { name: 'id', type: 'string', required: true },
          { name: 'title', type: 'string', required: false },
          { name: 'state_id', type: 'string', required: false },
          { name: 'priority', type: 'integer', required: false },
        ],
      },
      {
        name: 'linear.comments',
        description: 'Get comments on an issue',
        params: [{ name: 'issue_id', type: 'string', required: true }],
      },
      {
        name: 'linear.add_comment',
        description: 'Add a comment to an issue',
        params: [
          { name: 'issue_id', type: 'string', required: true },
          { name: 'body', type: 'string', required: true },
        ],
      },
      { name: 'linear.projects', description: 'List projects' },
      { name: 'linear.cycles', description: 'List cycles/sprints' },
      { name: 'linear.search', description: 'Global search across issues and projects' },
      { name: 'linear.states', description: 'List workflow states for a team' },
    ],
    added_at: '2026-01-15',
    updated_at: '2026-01-15',
    tier: 'free',
    download_count: 0,
  },
  {
    name: 'notion',
    version: '0.1.0',
    description: 'Notion pages, databases, and blocks - search, query, create, update',
    repository: 'https://github.com/fast-gateway-protocol/notion',
    license: 'MIT',
    platforms: ['darwin', 'linux'],
    categories: ['productivity', 'database'],
    featured: true,
    verified: true,
    skills: ['claude-code', 'cursor', 'windsurf'],
    auth: {
      type: 'bearer_token',
      provider: 'notion.so',
      setup: 'Set NOTION_API_KEY environment variable (create integration at notion.so/my-integrations)',
    },
    methods: [
      { name: 'notion.me', description: 'Get current bot/integration info' },
      { name: 'notion.users', description: 'List workspace users' },
      {
        name: 'notion.search',
        description: 'Search pages and databases',
        params: [
          { name: 'query', type: 'string', required: false },
          { name: 'filter', type: 'string', required: false },
          { name: 'limit', type: 'integer', required: false, default: 25 },
        ],
      },
      {
        name: 'notion.page',
        description: 'Get a page by ID',
        params: [{ name: 'page_id', type: 'string', required: true }],
      },
      {
        name: 'notion.database',
        description: 'Get a database schema by ID',
        params: [{ name: 'database_id', type: 'string', required: true }],
      },
      {
        name: 'notion.query_database',
        description: 'Query database rows with filters and sorts',
        params: [
          { name: 'database_id', type: 'string', required: true },
          { name: 'filter', type: 'object', required: false },
          { name: 'sorts', type: 'array', required: false },
          { name: 'limit', type: 'integer', required: false, default: 25 },
        ],
      },
      {
        name: 'notion.blocks',
        description: 'Get page/block children',
        params: [
          { name: 'block_id', type: 'string', required: true },
          { name: 'recursive', type: 'boolean', required: false, default: false },
        ],
      },
      {
        name: 'notion.create_page',
        description: 'Create a page in a database',
        params: [
          { name: 'database_id', type: 'string', required: true },
          { name: 'properties', type: 'object', required: true },
          { name: 'children', type: 'array', required: false },
        ],
      },
      {
        name: 'notion.update_page',
        description: 'Update page properties',
        params: [
          { name: 'page_id', type: 'string', required: true },
          { name: 'properties', type: 'object', required: true },
        ],
      },
      {
        name: 'notion.append_blocks',
        description: 'Append blocks to a page',
        params: [
          { name: 'block_id', type: 'string', required: true },
          { name: 'children', type: 'array', required: true },
        ],
      },
      {
        name: 'notion.comments',
        description: 'Get comments on a page/block',
        params: [{ name: 'block_id', type: 'string', required: true }],
      },
      {
        name: 'notion.add_comment',
        description: 'Add a comment to a page',
        params: [
          { name: 'page_id', type: 'string', required: true },
          { name: 'text', type: 'string', required: true },
        ],
      },
    ],
    added_at: '2026-01-15',
    updated_at: '2026-01-15',
    tier: 'free',
    download_count: 0,
  },
  // Example Paid Skills (for demonstration purposes - all official FGP skills are free)
  {
    name: 'example-twitter-research',
    version: '1.0.0',
    description: '[EXAMPLE LISTING] Twitter/X research automation - demonstrates how paid skills appear in the marketplace',
    repository: 'https://github.com/fast-gateway-protocol/examples',
    license: 'Example',
    platforms: ['darwin', 'linux'],
    categories: ['productivity', 'research'],
    featured: false,
    verified: false,
    skills: ['claude-code', 'cursor'],
    auth: {
      type: 'oauth2',
      provider: 'twitter',
      scopes: ['tweet.read', 'users.read'],
    },
    methods: [
      { name: 'twitter.search', description: 'Search tweets with filters' },
      { name: 'twitter.sentiment', description: 'Analyze sentiment' },
      { name: 'twitter.trends', description: 'Get trending topics' },
    ],
    added_at: '2026-01-10',
    updated_at: '2026-01-14',
    price_cents: 999,
    currency: 'USD',
    tier: 'community',
    seller: {
      name: 'Example Seller',
    },
    download_count: 0,
    rating: {
      average: 4.5,
      count: 3,
    },
  },
  {
    name: 'example-slack-tools',
    version: '1.0.0',
    description: '[EXAMPLE LISTING] Slack workspace tools - demonstrates Community tier pricing',
    repository: 'https://github.com/fast-gateway-protocol/examples',
    license: 'Example',
    platforms: ['darwin', 'linux'],
    categories: ['productivity', 'automation'],
    featured: false,
    verified: false,
    skills: ['claude-code'],
    auth: {
      type: 'oauth2',
      provider: 'slack',
      scopes: ['channels:read', 'chat:write'],
    },
    methods: [
      { name: 'slack.channels', description: 'List channels' },
      { name: 'slack.messages', description: 'Read messages' },
    ],
    added_at: '2026-01-12',
    updated_at: '2026-01-14',
    price_cents: 499,
    currency: 'USD',
    tier: 'community',
    seller: {
      name: 'Example Seller',
    },
    download_count: 0,
  },
];

export const categories = [
  { id: 'browser', name: 'Browser Automation', icon: 'globe' },
  { id: 'productivity', name: 'Productivity', icon: 'zap' },
  { id: 'email', name: 'Email', icon: 'mail' },
  { id: 'calendar', name: 'Calendar', icon: 'calendar' },
  { id: 'devtools', name: 'Developer Tools', icon: 'code' },
  { id: 'cloud', name: 'Cloud Services', icon: 'cloud' },
  { id: 'database', name: 'Database', icon: 'database' },
  { id: 'travel', name: 'Travel', icon: 'plane' },
  { id: 'research', name: 'Research', icon: 'search' },
  { id: 'automation', name: 'Automation', icon: 'workflow' },
];

// Helper functions for marketplace
export function formatPrice(priceCents: number | undefined, currency: string = 'USD'): string {
  if (!priceCents || priceCents === 0) return 'Free';
  const dollars = priceCents / 100;
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
  }).format(dollars);
}

export function getTierColor(tier: string | undefined): string {
  switch (tier) {
    case 'free':
      return 'var(--color-success)';
    case 'community':
      return 'var(--color-accent-secondary)';
    case 'verified':
      return 'var(--color-accent)';
    case 'pro':
      return '#a855f7';
    default:
      return 'var(--color-success)';
  }
}

export function getTierLabel(tier: string | undefined): string {
  switch (tier) {
    case 'free':
      return 'Free';
    case 'community':
      return 'Community';
    case 'verified':
      return 'Verified';
    case 'pro':
      return 'Pro';
    default:
      return 'Free';
  }
}

export function getPackage(name: string): Package | undefined {
  return packages.find((p) => p.name === name);
}

export function getPackagesByCategory(category: string): Package[] {
  return packages.filter((p) => p.categories.includes(category));
}

export function getFeaturedPackages(): Package[] {
  return packages.filter((p) => p.featured);
}

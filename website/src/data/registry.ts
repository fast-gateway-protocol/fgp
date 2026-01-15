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
}

export const packages: Package[] = [
  {
    name: 'browser',
    version: '0.1.0',
    description: 'Browser automation via Chrome DevTools Protocol - 292x faster than Playwright MCP',
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
      avg_latency_ms: 8,
      vs_mcp_speedup: '292x',
    },
    added_at: '2025-12-01',
    updated_at: '2026-01-14',
  },
  {
    name: 'gmail',
    version: '1.0.0',
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
      vs_mcp_speedup: '69x',
    },
    added_at: '2025-11-15',
    updated_at: '2026-01-10',
  },
  {
    name: 'calendar',
    version: '1.0.0',
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
    version: '1.0.0',
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
];

export function getPackage(name: string): Package | undefined {
  return packages.find((p) => p.name === name);
}

export function getPackagesByCategory(category: string): Package[] {
  return packages.filter((p) => p.categories.includes(category));
}

export function getFeaturedPackages(): Package[] {
  return packages.filter((p) => p.featured);
}

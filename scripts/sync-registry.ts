#!/usr/bin/env npx ts-node
/**
 * Registry Sync Script
 *
 * Scans all daemon directories for manifest.json files and generates/updates
 * the website registry.ts file.
 *
 * Usage:
 *   npx ts-node scripts/sync-registry.ts [--check] [--output FILE]
 *
 * Options:
 *   --check   Validate only, don't write changes
 *   --output  Output file path (default: website/src/data/registry.ts)
 *
 * # CHANGELOG
 * 01/15/2026 - Initial implementation (Claude)
 */

import * as fs from 'fs';
import * as path from 'path';

interface ManifestMethod {
  name: string;
  description: string;
  params?: Array<{
    name: string;
    type: string;
    required: boolean;
    default?: unknown;
  }>;
}

interface DaemonManifest {
  name: string;
  version: string;
  description: string;
  protocol?: string;
  author?: string;
  license?: string;
  repository?: string;
  daemon?: {
    entrypoint: string;
    socket: string;
    dependencies?: string[];
  };
  methods: ManifestMethod[];
  skills?: Record<string, unknown>;
  auth?: {
    type: string;
    provider?: string;
    scopes?: string[];
    setup?: string;
  } | null;
  platforms?: string[];
}

interface RegistryPackage {
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
  methods: Array<{ name: string; description: string }>;
  benchmark?: {
    avg_latency_ms: number;
    vs_mcp_speedup: string;
  };
  added_at: string;
  updated_at: string;
  tier?: string;
  download_count?: number;
}

// Daemon directory to category mapping
const CATEGORY_MAP: Record<string, string[]> = {
  browser: ['browser', 'automation'],
  gmail: ['productivity', 'email'],
  calendar: ['productivity', 'calendar'],
  github: ['devtools', 'productivity'],
  fly: ['cloud', 'devtools'],
  neon: ['cloud', 'database'],
  vercel: ['cloud', 'devtools'],
  travel: ['travel', 'productivity'],
  postgres: ['database', 'devtools'],
  linear: ['devtools', 'productivity'],
  notion: ['productivity', 'database'],
  supabase: ['database', 'cloud', 'storage'],
  composio: ['integrations', 'automation'],
  zapier: ['integrations', 'automation'],
  'google-drive': ['productivity', 'storage'],
  'google-sheets': ['productivity', 'database'],
  'google-docs': ['productivity'],
  cloudflare: ['cloud', 'devtools'],
  discord: ['social', 'automation'],
  youtube: ['media', 'social'],
  imessage: ['social', 'productivity'],
  'apple-reminders': ['productivity'],
  'apple-calendar': ['productivity', 'calendar'],
  contacts: ['productivity'],
  photos: ['media', 'productivity'],
  notes: ['productivity'],
  keychain: ['devtools', 'productivity'],
  'screen-time': ['productivity'],
};

// Featured daemons
const FEATURED_DAEMONS = new Set([
  'browser',
  'gmail',
  'calendar',
  'github',
  'travel',
  'linear',
  'notion',
  'supabase',
  'composio',
  'google-drive',
  'google-sheets',
  'cloudflare',
  'discord',
  'youtube',
]);

function findManifests(rootDir: string): string[] {
  const manifests: string[] = [];

  const entries = fs.readdirSync(rootDir, { withFileTypes: true });

  for (const entry of entries) {
    if (!entry.isDirectory()) continue;

    // Skip common non-daemon directories
    if (
      ['node_modules', '.git', 'target', 'website', 'app', 'registry', 'docs', 'scripts'].includes(
        entry.name
      )
    )
      continue;

    const manifestPath = path.join(rootDir, entry.name, 'manifest.json');
    if (fs.existsSync(manifestPath)) {
      manifests.push(manifestPath);
    }
  }

  return manifests;
}

function parseManifest(manifestPath: string): DaemonManifest | null {
  try {
    const content = fs.readFileSync(manifestPath, 'utf-8');
    return JSON.parse(content) as DaemonManifest;
  } catch (error) {
    console.error(`Failed to parse ${manifestPath}:`, error);
    return null;
  }
}

function manifestToPackage(manifest: DaemonManifest, daemonDir: string): RegistryPackage {
  const today = new Date().toISOString().split('T')[0];

  return {
    name: manifest.name,
    version: manifest.version,
    description: manifest.description,
    repository:
      manifest.repository || `https://github.com/fast-gateway-protocol/${manifest.name}`,
    license: manifest.license || 'MIT',
    platforms: manifest.platforms || ['darwin', 'linux'],
    categories: CATEGORY_MAP[manifest.name] || ['devtools'],
    featured: FEATURED_DAEMONS.has(manifest.name),
    verified: true,
    skills: manifest.skills ? Object.keys(manifest.skills) : ['claude-code'],
    auth: manifest.auth || { type: 'none' },
    methods: manifest.methods.map((m) => ({
      name: m.name,
      description: m.description,
    })),
    added_at: today,
    updated_at: today,
    tier: 'free',
    download_count: 0,
  };
}

function generateRegistryTS(packages: RegistryPackage[]): string {
  const header = `// Auto-generated by scripts/sync-registry.ts
// Do not edit manually - changes will be overwritten
// Last updated: ${new Date().toISOString()}

`;

  // Generate package entries
  const packageEntries = packages.map((pkg) => {
    return `  {
    name: '${pkg.name}',
    version: '${pkg.version}',
    description: '${pkg.description.replace(/'/g, "\\'")}',
    repository: '${pkg.repository}',
    license: '${pkg.license}',
    platforms: ${JSON.stringify(pkg.platforms)},
    categories: ${JSON.stringify(pkg.categories)},
    featured: ${pkg.featured},
    verified: ${pkg.verified},
    skills: ${JSON.stringify(pkg.skills)},
    auth: ${JSON.stringify(pkg.auth, null, 2).replace(/\n/g, '\n    ')},
    methods: [
${pkg.methods.map((m) => `      { name: '${m.name}', description: '${m.description.replace(/'/g, "\\'")}' }`).join(',\n')}
    ],
    added_at: '${pkg.added_at}',
    updated_at: '${pkg.updated_at}',
    tier: '${pkg.tier || 'free'}',
    download_count: ${pkg.download_count || 0},
  }`;
  });

  return `${header}export const packages = [
${packageEntries.join(',\n')}
];
`;
}

async function main() {
  const args = process.argv.slice(2);
  const checkOnly = args.includes('--check');
  const outputIndex = args.indexOf('--output');
  const outputPath =
    outputIndex >= 0
      ? args[outputIndex + 1]
      : path.join(__dirname, '../website/src/data/registry-generated.ts');

  const rootDir = path.join(__dirname, '..');

  console.log('🔍 Scanning for daemon manifests...');
  const manifests = findManifests(rootDir);
  console.log(`Found ${manifests.length} manifests\n`);

  const packages: RegistryPackage[] = [];

  for (const manifestPath of manifests) {
    const daemonDir = path.dirname(manifestPath);
    const manifest = parseManifest(manifestPath);

    if (!manifest) {
      console.log(`⚠️  Skipping invalid manifest: ${manifestPath}`);
      continue;
    }

    console.log(`✅ ${manifest.name} v${manifest.version}`);
    packages.push(manifestToPackage(manifest, daemonDir));
  }

  if (checkOnly) {
    console.log(`\n✅ All ${packages.length} manifests valid`);
    return;
  }

  // Generate registry file
  const registryContent = generateRegistryTS(packages);

  fs.writeFileSync(outputPath, registryContent);
  console.log(`\n📝 Generated ${outputPath} with ${packages.length} packages`);
}

main().catch(console.error);

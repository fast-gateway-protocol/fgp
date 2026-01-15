import { createFileRoute, Link } from '@tanstack/react-router';
import { useState, useMemo } from 'react';
import { motion } from 'framer-motion';
import { Search, Globe, Mail, Calendar, Github, Cloud, Database, Rocket, Zap, Check } from 'lucide-react';
import Fuse from 'fuse.js';
import { packages, categories } from '@/data/registry';
import type { Package } from '@/data/registry';

const iconMap: Record<string, React.ElementType> = {
  browser: Globe,
  gmail: Mail,
  calendar: Calendar,
  github: Github,
  fly: Rocket,
  neon: Database,
  vercel: Cloud,
  globe: Globe,
  mail: Mail,
  code: Github,
  cloud: Cloud,
  database: Database,
  zap: Zap,
};

function PackageCard({ pkg }: { pkg: Package }) {
  const Icon = iconMap[pkg.name] || Zap;

  return (
    <Link
      to="/marketplace/$packageName"
      params={{ packageName: pkg.name }}
      className="card group hover:border-[var(--color-accent)]/30 block"
    >
      <div className="flex items-start gap-4">
        <div className="p-3 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)] flex-shrink-0">
          <Icon className="w-6 h-6" />
        </div>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-semibold text-lg group-hover:text-[var(--color-accent)] transition-colors">
              {pkg.name}
            </h3>
            {pkg.verified && (
              <span className="flex items-center justify-center w-4 h-4 rounded-full bg-[var(--color-accent)] text-[var(--color-void)]">
                <Check className="w-3 h-3" />
              </span>
            )}
          </div>
          <p className="text-sm text-[var(--color-text-muted)] mt-1 line-clamp-2">
            {pkg.description}
          </p>
          <div className="flex items-center gap-2 mt-3 flex-wrap">
            {pkg.benchmark && (
              <span className="text-xs px-2 py-1 rounded-full bg-[var(--color-accent-secondary-muted)] text-[var(--color-accent-secondary)]">
                {pkg.benchmark.vs_mcp_speedup} faster
              </span>
            )}
            <span className="text-xs text-[var(--color-text-muted)]">
              {pkg.methods.length} methods
            </span>
            <span className="text-xs text-[var(--color-text-muted)]">
              v{pkg.version}
            </span>
          </div>
        </div>
      </div>
    </Link>
  );
}

function MarketplacePage() {
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedCategory, setSelectedCategory] = useState<string | null>(null);
  const [selectedPlatforms, setSelectedPlatforms] = useState<string[]>([]);

  const fuse = useMemo(
    () =>
      new Fuse(packages, {
        keys: [
          { name: 'name', weight: 3 },
          { name: 'description', weight: 1 },
          { name: 'methods.name', weight: 0.5 },
          { name: 'methods.description', weight: 0.3 },
        ],
        threshold: 0.3,
        ignoreLocation: true,
      }),
    []
  );

  const filteredPackages = useMemo(() => {
    let result = packages;

    // Search filter
    if (searchQuery.trim()) {
      result = fuse.search(searchQuery).map((r) => r.item);
    }

    // Category filter
    if (selectedCategory) {
      result = result.filter((p) => p.categories.includes(selectedCategory));
    }

    // Platform filter
    if (selectedPlatforms.length > 0) {
      result = result.filter((p) =>
        selectedPlatforms.every((platform) => p.platforms.includes(platform))
      );
    }

    return result;
  }, [searchQuery, selectedCategory, selectedPlatforms, fuse]);

  const togglePlatform = (platform: string) => {
    setSelectedPlatforms((prev) =>
      prev.includes(platform) ? prev.filter((p) => p !== platform) : [...prev, platform]
    );
  };

  return (
    <div className="section">
      <div className="container">
        <motion.div
          className="mb-12"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <h1 className="text-section">FGP Packages</h1>
          <p className="text-[var(--color-text-secondary)] mt-2">
            Browse and install fast daemon packages for your AI agents
          </p>
        </motion.div>

        <div className="flex flex-col lg:flex-row gap-8">
          {/* Sidebar Filters */}
          <motion.aside
            className="lg:w-64 flex-shrink-0"
            initial={{ opacity: 0, x: -20 }}
            animate={{ opacity: 1, x: 0 }}
            transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
          >
            {/* Search */}
            <div className="relative mb-6">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-[var(--color-text-muted)]" />
              <input
                type="text"
                placeholder="Search packages..."
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="input pl-10"
              />
            </div>

            {/* Categories */}
            <div className="mb-6">
              <h3 className="text-sm font-medium text-[var(--color-text-muted)] mb-3">Categories</h3>
              <div className="space-y-1">
                <button
                  onClick={() => setSelectedCategory(null)}
                  className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors ${
                    selectedCategory === null
                      ? 'bg-[var(--color-accent-muted)] text-[var(--color-accent)]'
                      : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]'
                  }`}
                >
                  All Packages
                </button>
                {categories.map((category) => {
                  const Icon = iconMap[category.icon] || Zap;
                  const count = packages.filter((p) => p.categories.includes(category.id)).length;
                  return (
                    <button
                      key={category.id}
                      onClick={() => setSelectedCategory(category.id)}
                      className={`w-full text-left px-3 py-2 rounded-lg text-sm transition-colors flex items-center gap-2 ${
                        selectedCategory === category.id
                          ? 'bg-[var(--color-accent-muted)] text-[var(--color-accent)]'
                          : 'text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]'
                      }`}
                    >
                      <Icon className="w-4 h-4" />
                      <span className="flex-1">{category.name}</span>
                      <span className="text-xs text-[var(--color-text-muted)]">{count}</span>
                    </button>
                  );
                })}
              </div>
            </div>

            {/* Platforms */}
            <div className="mb-6">
              <h3 className="text-sm font-medium text-[var(--color-text-muted)] mb-3">Platforms</h3>
              <div className="space-y-2">
                {['darwin', 'linux'].map((platform) => (
                  <label key={platform} className="flex items-center gap-2 cursor-pointer min-h-12">
                    <input
                      type="checkbox"
                      checked={selectedPlatforms.includes(platform)}
                      onChange={() => togglePlatform(platform)}
                      className="w-4 h-4 rounded border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                    />
                    <span className="text-sm text-[var(--color-text-secondary)]">
                      {platform === 'darwin' ? 'macOS' : 'Linux'}
                    </span>
                  </label>
                ))}
              </div>
            </div>
          </motion.aside>

          {/* Package Grid */}
          <div className="flex-1">
            <div className="grid md:grid-cols-2 gap-4">
              {filteredPackages.map((pkg, i) => (
                <motion.div
                  key={pkg.name}
                  initial={{ opacity: 0, y: 20 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ duration: 0.4, delay: i * 0.05, ease: [0.16, 1, 0.3, 1] }}
                >
                  <PackageCard pkg={pkg} />
                </motion.div>
              ))}
            </div>

            {filteredPackages.length === 0 && (
              <div className="text-center py-12">
                <p className="text-[var(--color-text-muted)]">No packages found matching your criteria</p>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

export const Route = createFileRoute('/marketplace/')({
  component: MarketplacePage,
});

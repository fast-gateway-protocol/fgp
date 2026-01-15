import { createFileRoute, Link, notFound } from '@tanstack/react-router';
import { useState } from 'react';
import { motion } from 'framer-motion';
import {
  Copy,
  Check,
  ArrowLeft,
  ExternalLink,
  Globe,
  Mail,
  Calendar,
  Github,
  Cloud,
  Database,
  Rocket,
  Zap,
  Shield,
  Terminal,
} from 'lucide-react';
import { getPackage } from '@/data/registry';
import type { Package, PackageMethod } from '@/data/registry';

const iconMap: Record<string, React.ElementType> = {
  browser: Globe,
  gmail: Mail,
  calendar: Calendar,
  github: Github,
  fly: Rocket,
  neon: Database,
  vercel: Cloud,
};

function InstallCommand({ command, label }: { command: string; label?: string }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div>
      {label && <div className="text-xs text-[var(--color-text-muted)] mb-2">{label}</div>}
      <div className="code-block flex items-center justify-between gap-4">
        <code>
          <span className="prompt">$ </span>
          <span className="command">{command}</span>
        </code>
        <button
          onClick={copy}
          className="flex-shrink-0 p-2 rounded-md hover:bg-[var(--color-surface-hover)] transition-colors text-[var(--color-text-muted)] hover:text-[var(--color-accent)]"
          aria-label="Copy to clipboard"
        >
          {copied ? <Check className="w-4 h-4 text-[var(--color-success)]" /> : <Copy className="w-4 h-4" />}
        </button>
      </div>
    </div>
  );
}

function MethodCard({ method }: { method: PackageMethod }) {
  const [expanded, setExpanded] = useState(false);

  return (
    <div className="border border-[var(--color-border)] rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        aria-expanded={expanded}
        className="w-full px-4 py-3 flex items-center justify-between text-left hover:bg-[var(--color-surface-hover)] transition-colors"
      >
        <div>
          <code className="text-[var(--color-accent)] text-sm">{method.name}</code>
          <p className="text-sm text-[var(--color-text-muted)] mt-0.5">{method.description}</p>
        </div>
        {method.params && method.params.length > 0 && (
          <span className="text-xs text-[var(--color-text-muted)] px-2 py-1 bg-[var(--color-surface)] rounded">
            {method.params.length} param{method.params.length !== 1 ? 's' : ''}
          </span>
        )}
      </button>
      {expanded && method.params && method.params.length > 0 && (
        <div className="px-4 pb-4 border-t border-[var(--color-border)]">
          <table className="w-full text-sm mt-3">
            <thead>
              <tr className="text-left text-[var(--color-text-muted)]">
                <th className="pb-2 font-medium">Parameter</th>
                <th className="pb-2 font-medium">Type</th>
                <th className="pb-2 font-medium">Required</th>
                <th className="pb-2 font-medium">Default</th>
              </tr>
            </thead>
            <tbody>
              {method.params.map((param) => (
                <tr key={param.name} className="border-t border-[var(--color-border)]">
                  <td className="py-2">
                    <code className="text-[var(--color-accent)]">{param.name}</code>
                  </td>
                  <td className="py-2 text-[var(--color-text-secondary)]">{param.type}</td>
                  <td className="py-2">
                    {param.required ? (
                      <span className="text-[var(--color-error)]">yes</span>
                    ) : (
                      <span className="text-[var(--color-text-muted)]">no</span>
                    )}
                  </td>
                  <td className="py-2 text-[var(--color-text-muted)]">
                    {param.default !== undefined ? String(param.default) : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  );
}

function PackageDetailPage({ pkg }: { pkg: Package }) {
  const Icon = iconMap[pkg.name] || Zap;

  return (
    <div className="section">
      <div className="container max-w-5xl">
        {/* Back link */}
        <Link
          to="/marketplace"
          className="inline-flex items-center gap-2 text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] transition-colors mb-8"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Marketplace
        </Link>

        {/* Header */}
        <motion.div
          className="card mb-8"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex flex-col md:flex-row md:items-start gap-6">
            <div className="p-4 rounded-xl bg-[var(--color-accent-muted)] text-[var(--color-accent)] flex-shrink-0">
              <Icon className="w-10 h-10" />
            </div>
            <div className="flex-1">
              <div className="flex items-center gap-3 flex-wrap">
                <h1 className="text-2xl font-bold">{pkg.name}</h1>
                <span className="text-sm text-[var(--color-text-muted)] px-2 py-1 bg-[var(--color-surface-elevated)] rounded">
                  v{pkg.version}
                </span>
                {pkg.verified && (
                  <span className="flex items-center gap-1 text-xs text-[var(--color-accent)] px-2 py-1 bg-[var(--color-accent-muted)] rounded">
                    <Check className="w-3 h-3" />
                    Verified
                  </span>
                )}
              </div>
              <p className="text-[var(--color-text-secondary)] mt-2">{pkg.description}</p>

              {/* Quick stats */}
              <div className="flex items-center gap-4 mt-4 flex-wrap">
                {pkg.benchmark && (
                  <div className="flex items-center gap-2">
                    <Zap className="w-4 h-4 text-[var(--color-accent-secondary)]" />
                    <span className="text-sm">
                      <span className="text-[var(--color-accent-secondary)] font-medium">
                        {pkg.benchmark.vs_mcp_speedup}
                      </span>{' '}
                      faster than MCP
                    </span>
                  </div>
                )}
                <div className="flex items-center gap-2">
                  <Terminal className="w-4 h-4 text-[var(--color-text-muted)]" />
                  <span className="text-sm text-[var(--color-text-muted)]">
                    {pkg.methods.length} methods
                  </span>
                </div>
              </div>

              {/* Badges */}
              <div className="flex items-center gap-2 mt-4 flex-wrap">
                {pkg.platforms.map((platform) => (
                  <span
                    key={platform}
                    className="text-xs px-2 py-1 rounded bg-[var(--color-surface-elevated)] text-[var(--color-text-muted)]"
                  >
                    {platform === 'darwin' ? 'macOS' : platform}
                  </span>
                ))}
                {pkg.skills.map((skill) => (
                  <span
                    key={skill}
                    className="text-xs px-2 py-1 rounded bg-[var(--color-surface-elevated)] text-[var(--color-text-muted)]"
                  >
                    {skill}
                  </span>
                ))}
              </div>
            </div>
          </div>

          {/* Install command */}
          <div className="mt-6 pt-6 border-t border-[var(--color-border)]">
            <InstallCommand command={`fgp install ${pkg.name}`} />
          </div>
        </motion.div>

        <div className="grid lg:grid-cols-3 gap-8">
          {/* Main content */}
          <div className="lg:col-span-2 space-y-8">
            {/* Methods */}
            <motion.section
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
            >
              <h2 className="text-xl font-semibold mb-4">Methods</h2>
              <div className="space-y-2">
                {pkg.methods.map((method) => (
                  <MethodCard key={method.name} method={method} />
                ))}
              </div>
            </motion.section>
          </div>

          {/* Sidebar */}
          <div className="space-y-6">
            {/* Quick Start */}
            <motion.section
              className="card"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
            >
              <h3 className="font-semibold mb-4">Quick Start</h3>
              <div className="space-y-4">
                <InstallCommand command={`fgp install ${pkg.name}`} label="Install" />
                <InstallCommand command={`fgp start ${pkg.name}`} label="Start daemon" />
                <InstallCommand
                  command={`fgp call ${pkg.methods[0]?.name || pkg.name + '.status'}`}
                  label="Test it"
                />
              </div>
            </motion.section>

            {/* Authentication */}
            <motion.section
              className="card"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
            >
              <h3 className="font-semibold mb-4 flex items-center gap-2">
                <Shield className="w-4 h-4 text-[var(--color-text-muted)]" />
                Authentication
              </h3>
              <div className="space-y-3 text-sm">
                <div className="flex justify-between">
                  <span className="text-[var(--color-text-muted)]">Type</span>
                  <span className="text-[var(--color-text-secondary)]">{pkg.auth.type}</span>
                </div>
                {pkg.auth.provider && (
                  <div className="flex justify-between">
                    <span className="text-[var(--color-text-muted)]">Provider</span>
                    <span className="text-[var(--color-text-secondary)]">{pkg.auth.provider}</span>
                  </div>
                )}
                {pkg.auth.setup && (
                  <div className="pt-2 border-t border-[var(--color-border)]">
                    <span className="text-[var(--color-text-muted)]">Setup:</span>
                    <code className="block mt-1 text-xs bg-[var(--color-surface-elevated)] p-2 rounded">
                      {pkg.auth.setup}
                    </code>
                  </div>
                )}
                {pkg.auth.scopes && (
                  <div className="pt-2 border-t border-[var(--color-border)]">
                    <span className="text-[var(--color-text-muted)]">Required scopes:</span>
                    <ul className="mt-2 space-y-1">
                      {pkg.auth.scopes.map((scope) => (
                        <li
                          key={scope}
                          className="text-xs text-[var(--color-text-secondary)] bg-[var(--color-surface-elevated)] p-1.5 rounded truncate"
                        >
                          {scope.split('/').pop()}
                        </li>
                      ))}
                    </ul>
                  </div>
                )}
              </div>
            </motion.section>

            {/* Links */}
            <motion.section
              className="card"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
            >
              <h3 className="font-semibold mb-4">Links</h3>
              <div className="space-y-2">
                <a
                  href={pkg.repository}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-2 text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-accent)] transition-colors"
                >
                  <Github className="w-4 h-4" />
                  View on GitHub
                  <ExternalLink className="w-3 h-3" />
                </a>
                <a
                  href={`${pkg.repository}/issues`}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="flex items-center gap-2 text-sm text-[var(--color-text-secondary)] hover:text-[var(--color-accent)] transition-colors"
                >
                  Report Issue
                  <ExternalLink className="w-3 h-3" />
                </a>
              </div>
            </motion.section>
          </div>
        </div>
      </div>
    </div>
  );
}

export const Route = createFileRoute('/marketplace/$packageName')({
  component: function PackageDetailWrapper() {
    const { packageName } = Route.useParams();
    const pkg = getPackage(packageName);

    if (!pkg) {
      throw notFound();
    }

    return <PackageDetailPage pkg={pkg} />;
  },
});

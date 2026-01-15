import { createFileRoute, Link } from '@tanstack/react-router';
import { Copy, Check, Zap, Globe, Mail, Calendar, Github, Cloud, Database, Rocket, ArrowRight } from 'lucide-react';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { packages } from '@/data/registry';
import { HeroSpeedup, BenchmarkChart, CumulativeOverhead } from '@/components/BenchmarkVisuals';

function InstallCommand({ command, compact = false }: { command: string; compact?: boolean }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className={`code-block ${compact ? 'compact' : ''} flex items-center justify-between gap-4`}>
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
  );
}

function StatCard({ value, label }: { value: string; label: string }) {
  return (
    <div className="stat-card">
      <div className="text-2xl md:text-3xl font-bold gradient-accent-text">{value}</div>
      <div className="text-sm text-[var(--color-text-muted)] mt-1">{label}</div>
    </div>
  );
}

const iconMap: Record<string, React.ElementType> = {
  browser: Globe,
  gmail: Mail,
  calendar: Calendar,
  github: Github,
  fly: Rocket,
  neon: Database,
  vercel: Cloud,
};

function PackageCard({ pkg }: { pkg: typeof packages[0] }) {
  const Icon = iconMap[pkg.name] || Zap;

  return (
    <Link
      to="/marketplace/$packageName"
      params={{ packageName: pkg.name }}
      className="card group hover:border-[var(--color-accent)]/30"
    >
      <div className="flex items-start gap-4">
        <div className="p-3 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
          <Icon className="w-6 h-6" />
        </div>
        <div className="flex-1 min-w-0">
          <h3 className="font-semibold text-lg group-hover:text-[var(--color-accent)] transition-colors">
            {pkg.name}
          </h3>
          <p className="text-sm text-[var(--color-text-muted)] mt-1 line-clamp-2">
            {pkg.description}
          </p>
          {pkg.benchmark && (
            <div className="flex items-center gap-2 mt-3">
              <span className="text-xs px-2 py-1 rounded-full bg-[var(--color-accent-secondary-muted)] text-[var(--color-accent-secondary)]">
                {pkg.benchmark.vs_mcp_speedup} faster
              </span>
            </div>
          )}
        </div>
      </div>
    </Link>
  );
}

function HomePage() {
  const featuredPackages = packages.filter(p => p.featured).slice(0, 4);

  return (
    <>
      {/* Hero Section */}
      <section className="section pt-24 md:pt-32 pb-20 overflow-hidden">
        <div className="container">
          <div className="hero-grid">
            <div className="space-y-8">
              <motion.div
                className="flex flex-wrap items-center gap-3"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
              >
                <span className="eyebrow">Fast Gateway Protocol</span>
                <span className="nav-pill">Open source</span>
              </motion.div>

              <motion.h1
                className="text-hero text-balance"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
              >
                Agent tooling that feels instant, not cold-started.
              </motion.h1>

              <motion.p
                className="text-lg md:text-xl text-[var(--color-text-secondary)] text-balance max-w-xl"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
              >
                FGP turns slow MCP stdio calls into always-on daemons with UNIX sockets. Get{' '}
                <span className="gradient-accent-text font-semibold">10-50ms latency</span> and keep
                agents focused on work, not startup overhead.
              </motion.p>

              <motion.div
                className="flex flex-wrap items-center gap-4"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.5, ease: [0.16, 1, 0.3, 1] }}
              >
                <Link to="/docs" className="btn btn-primary">
                  Install CLI
                </Link>
                <Link to="/marketplace" className="btn btn-secondary">
                  Browse daemons
                </Link>
              </motion.div>

              <motion.div
                className="hero-stats"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.6, ease: [0.16, 1, 0.3, 1] }}
              >
                <StatCard value="8ms" label="Browser navigate" />
                <StatCard value="19x" label="Workflow speedup" />
                <StatCard value="2.3s" label="MCP cold start" />
              </motion.div>

              <motion.div
                className="hero-chips"
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ duration: 0.6, delay: 0.7, ease: [0.16, 1, 0.3, 1] }}
              >
                <span className="chip">
                  <Zap className="w-3.5 h-3.5" />
                  Persistent daemons
                </span>
                <span className="chip">
                  <Rocket className="w-3.5 h-3.5" />
                  UNIX sockets
                </span>
                <span className="chip">
                  <Database className="w-3.5 h-3.5" />
                  Typed contracts
                </span>
              </motion.div>
            </div>

            <motion.div
              className="hero-panel space-y-6"
              initial={{ opacity: 0, y: 30 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.7, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
            >
              <div className="flex items-center justify-between">
                <span className="eyebrow">Benchmarks</span>
                <span className="text-sm text-[var(--color-text-muted)]">Real workflows</span>
              </div>

              <HeroSpeedup compact />

              <div className="divider-glow" />

              <div className="code-stack">
                <InstallCommand command="curl -sSL getfgp.com/install | bash" compact />
                <InstallCommand command="fgp install browser" compact />
              </div>

              <div className="hero-chips">
                <span className="chip">10-50ms latency</span>
                <span className="chip">Daemon lifecycle</span>
                <span className="chip">MCP compatible</span>
              </div>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Performance Section */}
      <section className="section">
        <div className="container">
          <div className="split-layout">
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
              className="space-y-6"
            >
              <div className="space-y-4">
                <span className="eyebrow">Performance</span>
                <h2 className="text-section text-balance">
                  Cold starts drag your agents. FGP keeps them warm.
                </h2>
                <p className="text-[var(--color-text-secondary)] text-balance">
                  Every MCP tool call spins up a new process. That latency compounds across a workflow.
                  FGP runs persistent daemons so responses feel immediate.
                </p>
              </div>

              <div className="section-panel space-y-4">
                <div className="flex items-center justify-between">
                  <span className="text-sm text-[var(--color-text-muted)]">Typical overhead</span>
                  <span className="text-sm text-[var(--color-text-muted)]">Per call</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-lg font-semibold">MCP stdio</span>
                  <span className="text-lg font-semibold text-red-400">~2.3s</span>
                </div>
                <div className="flex items-center justify-between">
                  <span className="text-lg font-semibold">FGP daemon</span>
                  <span className="text-lg font-semibold gradient-accent-text">10-50ms</span>
                </div>
              </div>
            </motion.div>

            <motion.div
              className="section-panel"
              initial={{ opacity: 0, y: 20 }}
              whileInView={{ opacity: 1, y: 0 }}
              viewport={{ once: true }}
              transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
            >
              <BenchmarkChart />
            </motion.div>
          </div>
        </div>
      </section>

      {/* Cumulative Overhead Section */}
      <section className="section bg-[var(--color-surface)]">
        <div className="container">
          <CumulativeOverhead />
        </div>
      </section>

      {/* How It Works */}
      <section className="section bg-[var(--color-surface)]">
        <div className="container">
          <motion.div
            className="text-center mb-12"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <h2 className="text-section">How It Works</h2>
          </motion.div>

          <div className="grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
            {[
              {
                step: '1',
                title: 'Install a daemon',
                code: 'fgp install gmail',
              },
              {
                step: '2',
                title: 'Start it',
                code: 'fgp start gmail',
              },
              {
                step: '3',
                title: 'Use from any agent',
                code: 'fgp call gmail.inbox',
              },
            ].map((item, i) => (
              <motion.div
                key={item.step}
                className="card space-y-4"
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: i * 0.1, ease: [0.16, 1, 0.3, 1] }}
              >
                <div className="flex items-center justify-between">
                  <span className="eyebrow">Step {item.step}</span>
                  <span className="chip">1 command</span>
                </div>
                <h3 className="text-lg font-semibold">{item.title}</h3>
                <div className="code-block compact text-left">
                  <code>
                    <span className="prompt">$ </span>
                    <span className="command">{item.code}</span>
                  </code>
                </div>
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Featured Packages */}
      <section className="section">
        <div className="container">
          <motion.div
            className="flex items-center justify-between mb-8"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <div>
              <span className="eyebrow">Marketplace</span>
              <h2 className="text-section mt-3">Featured Packages</h2>
            </div>
            <Link
              to="/marketplace"
              className="flex items-center gap-2 text-[var(--color-accent)] hover:underline"
            >
              Browse all <ArrowRight className="w-4 h-4" />
            </Link>
          </motion.div>

          <div className="grid md:grid-cols-2 gap-6">
            {featuredPackages.map((pkg, i) => (
              <motion.div
                key={pkg.name}
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: i * 0.1, ease: [0.16, 1, 0.3, 1] }}
              >
                <PackageCard pkg={pkg} />
              </motion.div>
            ))}
          </div>
        </div>
      </section>

      {/* Works With Section */}
      <section className="section bg-[var(--color-surface)]">
        <div className="container">
          <motion.div
            className="text-center"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <span className="eyebrow">Compatible</span>
            <h2 className="text-section mt-4 mb-12">Works With</h2>
            <div className="flex flex-wrap justify-center items-center gap-12 md:gap-20">
              {/* Claude Code */}
              <div className="group flex flex-col items-center gap-3">
                <div className="w-16 h-16 rounded-2xl bg-[#D97757]/10 flex items-center justify-center group-hover:bg-[#D97757]/20 transition-colors">
                  <svg viewBox="0 0 24 24" className="w-10 h-10" fill="none">
                    <path
                      d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 15h-2v-6h2v6zm4 0h-2V7h2v10z"
                      fill="#D97757"
                    />
                  </svg>
                </div>
                <span className="text-sm text-[var(--color-text-muted)] group-hover:text-[var(--color-text-secondary)] transition-colors">
                  Claude Code
                </span>
              </div>

              {/* Cursor */}
              <div className="group flex flex-col items-center gap-3">
                <div className="w-16 h-16 rounded-2xl bg-[#00D8FF]/10 flex items-center justify-center group-hover:bg-[#00D8FF]/20 transition-colors">
                  <svg viewBox="0 0 24 24" className="w-10 h-10" fill="none">
                    <path
                      d="M5 3l14 9-14 9V3z"
                      fill="#00D8FF"
                    />
                  </svg>
                </div>
                <span className="text-sm text-[var(--color-text-muted)] group-hover:text-[var(--color-text-secondary)] transition-colors">
                  Cursor
                </span>
              </div>

              {/* Windsurf */}
              <div className="group flex flex-col items-center gap-3">
                <div className="w-16 h-16 rounded-2xl bg-[#0EA5E9]/10 flex items-center justify-center group-hover:bg-[#0EA5E9]/20 transition-colors">
                  <svg viewBox="0 0 24 24" className="w-10 h-10" fill="none">
                    <path
                      d="M2 12c0 0 4-8 10-8s10 8 10 8-4 8-10 8-10-8-10-8z"
                      stroke="#0EA5E9"
                      strokeWidth="2"
                      fill="none"
                    />
                    <path
                      d="M6 12h12M12 6v12"
                      stroke="#0EA5E9"
                      strokeWidth="2"
                    />
                  </svg>
                </div>
                <span className="text-sm text-[var(--color-text-muted)] group-hover:text-[var(--color-text-secondary)] transition-colors">
                  Windsurf
                </span>
              </div>

              {/* Continue */}
              <div className="group flex flex-col items-center gap-3">
                <div className="w-16 h-16 rounded-2xl bg-[#FF6B6B]/10 flex items-center justify-center group-hover:bg-[#FF6B6B]/20 transition-colors">
                  <svg viewBox="0 0 24 24" className="w-10 h-10" fill="none">
                    <path
                      d="M8 5v14l11-7L8 5z"
                      fill="#FF6B6B"
                    />
                  </svg>
                </div>
                <span className="text-sm text-[var(--color-text-muted)] group-hover:text-[var(--color-text-secondary)] transition-colors">
                  Continue
                </span>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="section">
        <div className="container">
          <motion.div
            className="section-panel max-w-3xl mx-auto text-center space-y-6"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <span className="eyebrow">Get started</span>
            <h2 className="text-section">Install in minutes</h2>
            <p className="text-[var(--color-text-secondary)]">
              Install the FGP CLI and start using fast daemons in minutes.
            </p>
            <div className="max-w-md mx-auto code-stack">
              <InstallCommand command="curl -sSL getfgp.com/install | bash" compact />
              <InstallCommand command="fgp install browser" compact />
              <InstallCommand command="fgp start browser" compact />
            </div>
          </motion.div>
        </div>
      </section>
    </>
  );
}

export const Route = createFileRoute('/')({
  component: HomePage,
});

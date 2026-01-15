import { createFileRoute, Link } from '@tanstack/react-router';
import { Copy, Check, Zap, Globe, Mail, Calendar, Github, Cloud, Database, Rocket, ArrowRight } from 'lucide-react';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { packages } from '@/data/registry';
import { HeroSpeedup, BenchmarkChart, CumulativeOverhead } from '@/components/BenchmarkVisuals';

function InstallCommand({ command }: { command: string }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
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
  );
}

function StatCard({ value, label }: { value: string; label: string }) {
  return (
    <div className="text-center">
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
      <section className="section pt-24 md:pt-32 pb-16 overflow-hidden">
        <div className="container">
          <div className="max-w-4xl mx-auto text-center">
            {/* Big animated 292x */}
            <HeroSpeedup />

            <motion.p
              className="text-xl md:text-2xl text-[var(--color-text-secondary)] mt-8 max-w-2xl mx-auto"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
            >
              Daemon-based tools for AI agents. Replace slow MCP stdio servers with persistent
              UNIX socket daemons.{' '}
              <span className="gradient-accent-text font-semibold">10-50ms latency</span>, not 2.3 seconds.
            </motion.p>

            <motion.div
              className="max-w-md mx-auto mt-10"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.5, ease: [0.16, 1, 0.3, 1] }}
            >
              <InstallCommand command="curl -sSL getfgp.com/install | bash" />
            </motion.div>

            <motion.div
              className="flex flex-wrap justify-center gap-8 md:gap-16 mt-12"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.6, ease: [0.16, 1, 0.3, 1] }}
            >
              <StatCard value="8ms" label="Browser navigate" />
              <StatCard value="19x" label="Workflow speedup" />
              <StatCard value="MIT" label="License" />
            </motion.div>
          </div>
        </div>
      </section>

      {/* Performance Section */}
      <section className="section">
        <div className="container">
          <motion.div
            className="text-center mb-12"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <h2 className="text-section">
              The <span className="text-red-400">MCP</span> Problem
            </h2>
            <p className="text-[var(--color-text-secondary)] mt-4 max-w-2xl mx-auto">
              Every MCP tool call spawns a new process. Cold-start overhead compounds across workflows.
              FGP keeps daemons <span className="gradient-accent-text font-medium">warm and ready</span>.
            </p>
          </motion.div>

          {/* New animated benchmark chart */}
          <div className="max-w-3xl mx-auto">
            <BenchmarkChart />
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

          <div className="grid md:grid-cols-3 gap-8 max-w-4xl mx-auto">
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
                className="text-center"
                initial={{ opacity: 0, y: 20 }}
                whileInView={{ opacity: 1, y: 0 }}
                viewport={{ once: true }}
                transition={{ duration: 0.6, delay: i * 0.1, ease: [0.16, 1, 0.3, 1] }}
              >
                <div className="w-10 h-10 rounded-full gradient-accent text-[var(--color-void)] font-bold flex items-center justify-center mx-auto">
                  {item.step}
                </div>
                <h3 className="font-semibold mt-4">{item.title}</h3>
                <div className="code-block mt-3 text-left">
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
            <h2 className="text-section">Featured Packages</h2>
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
            <h2 className="text-section mb-12">Works With</h2>
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
            className="max-w-2xl mx-auto text-center"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <h2 className="text-section">Get Started</h2>
            <p className="text-[var(--color-text-secondary)] mt-4">
              Install the FGP CLI and start using fast daemons in minutes.
            </p>
            <div className="max-w-md mx-auto mt-8 space-y-4">
              <InstallCommand command="curl -sSL getfgp.com/install | bash" />
              <InstallCommand command="fgp install browser" />
              <InstallCommand command="fgp start browser" />
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

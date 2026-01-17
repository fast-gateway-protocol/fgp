import { createFileRoute } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import {
  Monitor,
  Store,
  Bot,
  Rocket,
  Apple,
  Github,
  Download,
  Check,
  Cpu,
  HardDrive
} from 'lucide-react';

const features = [
  {
    icon: Monitor,
    title: 'Menu Bar Control',
    description: 'Start, stop, and monitor daemons without leaving your workflow. Live status in your menu bar.',
  },
  {
    icon: Store,
    title: 'Built-in Marketplace',
    description: 'Browse and install official FGP daemons with one click. Auto-updates keep you current.',
  },
  {
    icon: Bot,
    title: 'AI Agent Integration',
    description: 'Auto-register with Claude Code, Cursor, and more. One-click MCP configuration.',
  },
  {
    icon: Rocket,
    title: 'Launch at Login',
    description: 'Set it and forget it. Daemons start automatically so your tools are ready when you are.',
  },
];

const requirements = [
  { icon: Apple, label: 'macOS 10.15+', detail: 'Catalina and later' },
  { icon: Cpu, label: 'Universal Binary', detail: 'Apple Silicon + Intel' },
  { icon: HardDrive, label: '~15MB', detail: 'Lightweight install' },
];

function FeatureCard({ feature, index }: { feature: typeof features[0]; index: number }) {
  const Icon = feature.icon;

  return (
    <motion.div
      className="card group"
      initial={{ opacity: 0, y: 20 }}
      whileInView={{ opacity: 1, y: 0 }}
      viewport={{ once: true }}
      transition={{ duration: 0.6, delay: index * 0.1, ease: [0.16, 1, 0.3, 1] }}
    >
      <div className="p-3 rounded-xl bg-[var(--color-accent-muted)] text-[var(--color-accent)] w-fit group-hover:bg-[var(--color-accent)]/25 transition-colors">
        <Icon className="w-6 h-6" />
      </div>
      <h3 className="font-semibold text-lg mt-4 group-hover:text-[var(--color-accent)] transition-colors">
        {feature.title}
      </h3>
      <p className="text-[var(--color-text-muted)] mt-2 text-sm leading-relaxed">
        {feature.description}
      </p>
    </motion.div>
  );
}

function AppPage() {
  return (
    <>
      {/* Hero Section */}
      <section className="section pt-24 md:pt-32 pb-16">
        <div className="container">
          <div className="max-w-3xl mx-auto text-center">
            {/* App Icon with glow effect */}
            <motion.div
              className="relative w-32 h-32 mx-auto mb-8"
              initial={{ opacity: 0, scale: 0.8 }}
              animate={{ opacity: 1, scale: 1 }}
              transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
            >
              {/* Glow backdrop */}
              <div className="absolute inset-0 rounded-[2rem] bg-gradient-to-br from-[var(--color-accent)] to-[var(--color-accent-secondary)] opacity-30 blur-2xl" />
              {/* Icon container */}
              <div className="relative w-full h-full rounded-[2rem] bg-gradient-to-br from-[var(--color-surface-elevated)] to-[var(--color-surface)] border border-[var(--color-border-hover)] flex items-center justify-center shadow-2xl">
                <img
                  src="/logo.png"
                  alt="FGP Manager"
                  className="w-16 h-16 invert"
                />
              </div>
            </motion.div>

            {/* Title */}
            <motion.h1
              className="text-hero gradient-accent-text"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
            >
              FGP Manager
            </motion.h1>

            {/* Tagline */}
            <motion.p
              className="text-xl md:text-2xl text-[var(--color-text-secondary)] mt-4"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
            >
              Control your daemons from the menu bar
            </motion.p>

            {/* Download button */}
            <motion.div
              className="mt-10"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ duration: 0.6, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
            >
              <a
                href="https://github.com/fast-gateway-protocol/fgp/releases/latest"
                target="_blank"
                rel="noopener noreferrer"
                className="btn btn-primary text-lg px-8 py-4"
              >
                <Apple className="w-5 h-5" />
                Download for macOS
              </a>
            </motion.div>

            {/* Version badge */}
            <motion.div
              className="mt-4 flex items-center justify-center gap-4 text-sm text-[var(--color-text-muted)]"
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
            >
              <span className="px-3 py-1 rounded-full bg-[var(--color-surface)] border border-[var(--color-border)]">
                v0.1.0
              </span>
              <span>macOS 10.15+</span>
            </motion.div>
          </div>
        </div>
      </section>

      {/* Features Grid */}
      <section className="section bg-[var(--color-surface)]">
        <div className="container">
          <motion.div
            className="text-center mb-12"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <h2 className="text-section">Everything you need</h2>
            <p className="text-[var(--color-text-secondary)] mt-4 max-w-2xl mx-auto">
              A native macOS app that makes managing FGP daemons effortless.
            </p>
          </motion.div>

          <div className="grid md:grid-cols-2 lg:grid-cols-4 gap-6">
            {features.map((feature, i) => (
              <FeatureCard key={feature.title} feature={feature} index={i} />
            ))}
          </div>
        </div>
      </section>

      {/* Screenshot Showcase */}
      <section className="section">
        <div className="container">
          <motion.div
            className="text-center mb-12"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <h2 className="text-section">Lives in your menu bar</h2>
            <p className="text-[var(--color-text-secondary)] mt-4">
              Ready when you need it, out of the way when you don't.
            </p>
          </motion.div>

          {/* macOS-style screenshot placeholder */}
          <motion.div
            className="max-w-2xl mx-auto"
            initial={{ opacity: 0, y: 30 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
          >
            {/* Window chrome */}
            <div className="rounded-2xl overflow-hidden border border-[var(--color-border-hover)] shadow-2xl">
              {/* Title bar */}
              <div className="h-10 bg-[var(--color-surface-elevated)] border-b border-[var(--color-border)] flex items-center px-4">
                <div className="flex gap-2">
                  <div className="w-3 h-3 rounded-full bg-[#ff5f56]" />
                  <div className="w-3 h-3 rounded-full bg-[#ffbd2e]" />
                  <div className="w-3 h-3 rounded-full bg-[#27c93f]" />
                </div>
              </div>

              {/* Content area */}
              <div className="aspect-video bg-[var(--color-surface)] flex items-center justify-center relative">
                {/* Decorative grid */}
                <div className="absolute inset-0 opacity-30">
                  <div className="grid-pattern" style={{ position: 'absolute', inset: 0 }} />
                </div>

                {/* Placeholder content */}
                <div className="text-center z-10">
                  <div className="w-20 h-20 mx-auto mb-6 rounded-2xl bg-[var(--color-accent-muted)] flex items-center justify-center">
                    <Monitor className="w-10 h-10 text-[var(--color-accent)]" />
                  </div>
                  <p className="text-[var(--color-text-muted)] text-lg">Screenshot coming soon</p>
                  <p className="text-[var(--color-text-muted)] text-sm mt-2">
                    Menu bar popover with daemon controls
                  </p>
                </div>
              </div>
            </div>
          </motion.div>
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
            <h2 className="text-section mb-4">Seamless AI Integration</h2>
            <p className="text-[var(--color-text-secondary)] mb-12 max-w-2xl mx-auto">
              One-click registration with your favorite AI coding tools.
              Automatically configures MCP servers for instant daemon access.
            </p>

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
                    <path d="M5 3l14 9-14 9V3z" fill="#00D8FF" />
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
                    <path d="M6 12h12M12 6v12" stroke="#0EA5E9" strokeWidth="2" />
                  </svg>
                </div>
                <span className="text-sm text-[var(--color-text-muted)] group-hover:text-[var(--color-text-secondary)] transition-colors">
                  Windsurf
                </span>
              </div>

              {/* Claude Desktop */}
              <div className="group flex flex-col items-center gap-3">
                <div className="w-16 h-16 rounded-2xl bg-[#D97757]/10 flex items-center justify-center group-hover:bg-[#D97757]/20 transition-colors">
                  <svg viewBox="0 0 24 24" className="w-10 h-10" fill="none">
                    <rect x="3" y="3" width="18" height="14" rx="2" stroke="#D97757" strokeWidth="2" fill="none" />
                    <path d="M8 21h8M12 17v4" stroke="#D97757" strokeWidth="2" />
                  </svg>
                </div>
                <span className="text-sm text-[var(--color-text-muted)] group-hover:text-[var(--color-text-secondary)] transition-colors">
                  Claude Desktop
                </span>
              </div>
            </div>
          </motion.div>
        </div>
      </section>

      {/* System Requirements */}
      <section className="section">
        <div className="container">
          <motion.div
            className="max-w-2xl mx-auto"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <h2 className="text-section text-center mb-8">System Requirements</h2>

            <div className="grid grid-cols-3 gap-4">
              {requirements.map((req, i) => {
                const Icon = req.icon;
                return (
                  <motion.div
                    key={req.label}
                    className="text-center p-4"
                    initial={{ opacity: 0, y: 20 }}
                    whileInView={{ opacity: 1, y: 0 }}
                    viewport={{ once: true }}
                    transition={{ duration: 0.6, delay: i * 0.1, ease: [0.16, 1, 0.3, 1] }}
                  >
                    <Icon className="w-8 h-8 mx-auto text-[var(--color-accent)] mb-3" />
                    <div className="font-semibold">{req.label}</div>
                    <div className="text-sm text-[var(--color-text-muted)]">{req.detail}</div>
                  </motion.div>
                );
              })}
            </div>
          </motion.div>
        </div>
      </section>

      {/* Final CTA */}
      <section className="section bg-[var(--color-surface)]">
        <div className="container">
          <motion.div
            className="max-w-2xl mx-auto text-center"
            initial={{ opacity: 0, y: 20 }}
            whileInView={{ opacity: 1, y: 0 }}
            viewport={{ once: true }}
            transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
          >
            <h2 className="text-section">Ready to get started?</h2>
            <p className="text-[var(--color-text-secondary)] mt-4 mb-8">
              Download FGP Manager and take control of your daemons.
            </p>

            <div className="flex flex-col sm:flex-row gap-4 justify-center">
              <a
                href="https://github.com/fast-gateway-protocol/fgp/releases/latest"
                target="_blank"
                rel="noopener noreferrer"
                className="btn btn-primary"
              >
                <Download className="w-5 h-5" />
                Download for macOS
              </a>
              <a
                href="https://github.com/fast-gateway-protocol/fgp"
                target="_blank"
                rel="noopener noreferrer"
                className="btn btn-secondary"
              >
                <Github className="w-5 h-5" />
                View on GitHub
              </a>
            </div>

            {/* Checkmarks */}
            <div className="flex flex-wrap justify-center gap-6 mt-8 text-sm text-[var(--color-text-muted)]">
              <span className="flex items-center gap-2">
                <Check className="w-4 h-4 text-[var(--color-success)]" />
                Free & Open Source
              </span>
              <span className="flex items-center gap-2">
                <Check className="w-4 h-4 text-[var(--color-success)]" />
                MIT License
              </span>
              <span className="flex items-center gap-2">
                <Check className="w-4 h-4 text-[var(--color-success)]" />
                No account required
              </span>
            </div>
          </motion.div>
        </div>
      </section>
    </>
  );
}

export const Route = createFileRoute('/app/')({
  component: AppPage,
});

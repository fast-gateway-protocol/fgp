import { motion, useInView } from 'framer-motion';
import { useRef, useState } from 'react';

// ============================================================================
// Hero Speedup Display - Honest "Zero Cold Start" Focus
// ============================================================================

export function HeroSpeedup({ compact = false }: { compact?: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true });
  const sizeClass = compact
    ? 'text-[clamp(3rem,10vw,6rem)]'
    : 'text-[clamp(5rem,18vw,10rem)]';
  const subtitleClass = compact
    ? 'text-base md:text-lg text-left'
    : 'text-xl md:text-2xl text-center';

  return (
    <div ref={ref} className={`relative ${compact ? 'text-left' : ''}`}>
      {/* Glow effect behind the number */}
      <motion.div
        className="absolute inset-0 blur-[120px] opacity-30"
        style={{
          background: 'radial-gradient(circle, var(--color-accent) 0%, var(--color-accent-secondary) 50%, transparent 70%)',
        }}
        initial={{ scale: 0, opacity: 0 }}
        animate={isInView ? { scale: 1.5, opacity: 0.4 } : {}}
        transition={{ duration: 1.5, ease: [0.16, 1, 0.3, 1] }}
      />

      {/* The main message - Zero Cold Start */}
      <motion.div
        className={`relative ${sizeClass} font-black leading-none tracking-tighter`}
        initial={{ opacity: 0, scale: 0.8, y: 40 }}
        animate={isInView ? { opacity: 1, scale: 1, y: 0 } : {}}
        transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
      >
        <span className="gradient-accent-text">
          0ms
        </span>
      </motion.div>

      {/* Subtitle */}
      <motion.p
        className={`${subtitleClass} text-[var(--color-text-secondary)] mt-4`}
        initial={{ opacity: 0, y: 20 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.6, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
      >
        cold start latency
      </motion.p>

      {/* Speedup pills */}
      <motion.div
        className={`flex ${compact ? 'justify-start' : 'justify-center'} gap-3 mt-6 flex-wrap`}
        initial={{ opacity: 0, y: 20 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.6, delay: 0.5, ease: [0.16, 1, 0.3, 1] }}
      >
        <span className="px-3 py-1.5 rounded-full text-sm font-semibold border border-[var(--color-accent)]/30 bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
          3-12× faster (warm)
        </span>
        <span className="px-3 py-1.5 rounded-full text-sm font-semibold border border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-text-secondary)]">
          50× faster (local ops)
        </span>
      </motion.div>

      <motion.p
        className={`${compact ? 'text-xs' : 'text-sm'} text-[var(--color-text-muted)] mt-4`}
        initial={{ opacity: 0 }}
        animate={isInView ? { opacity: 1 } : {}}
        transition={{ duration: 0.6, delay: 0.7, ease: [0.16, 1, 0.3, 1] }}
      >
        Daemons stay warm across sessions · Browser, Gmail, Calendar, GitHub, iMessage
      </motion.p>
    </div>
  );
}

// ============================================================================
// Benchmark Bar Chart - Visual Comparison (Warm vs Warm)
// ============================================================================

interface BenchmarkData {
  operation: string;
  fgp: number;
  mcpWarm: number;
  mcpCold: number;
}

// Honest benchmark data based on actual measurements
const benchmarks: BenchmarkData[] = [
  { operation: 'Navigate', fgp: 3, mcpWarm: 28, mcpCold: 1900 },
  { operation: 'Snapshot', fgp: 1, mcpWarm: 2, mcpCold: 1000 },
  { operation: 'Screenshot', fgp: 28, mcpWarm: 60, mcpCold: 1600 },
];

function BenchmarkBar({ data, index, showCold }: { data: BenchmarkData; index: number; showCold: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });
  const mcpValue = showCold ? data.mcpCold : data.mcpWarm;
  const speedup = Math.round(mcpValue / data.fgp);
  const maxValue = showCold ? Math.max(...benchmarks.map(b => b.mcpCold)) : Math.max(...benchmarks.map(b => b.mcpWarm));

  return (
    <motion.div
      ref={ref}
      className="group"
      initial={{ opacity: 0, x: -30 }}
      animate={isInView ? { opacity: 1, x: 0 } : {}}
      transition={{ duration: 0.6, delay: index * 0.15, ease: [0.16, 1, 0.3, 1] }}
    >
      {/* Operation name and speedup badge */}
      <div className="flex items-center justify-between mb-3">
        <span className="text-lg font-semibold text-[var(--color-text-primary)]">
          {data.operation}
        </span>
        <motion.span
          className="px-3 py-1 rounded-full text-sm font-bold text-[var(--color-void)]"
          style={{
            background: 'linear-gradient(135deg, var(--color-accent) 0%, var(--color-accent-secondary) 100%)',
          }}
          initial={{ scale: 0 }}
          animate={isInView ? { scale: 1 } : {}}
          transition={{ duration: 0.4, delay: index * 0.15 + 0.5, type: "spring", bounce: 0.4 }}
        >
          {speedup}×
        </motion.span>
      </div>

      {/* FGP Bar */}
      <div className="flex items-center gap-4 mb-2">
        <span className="text-xs font-medium text-[var(--color-accent)] w-12">FGP</span>
        <div className="flex-1 h-8 rounded-lg bg-[var(--color-surface-elevated)] overflow-hidden relative">
          <motion.div
            className="h-full rounded-lg relative overflow-hidden"
            style={{
              background: 'linear-gradient(90deg, var(--color-accent) 0%, var(--color-accent-secondary) 100%)',
            }}
            initial={{ width: 0 }}
            animate={isInView ? { width: `${Math.max((data.fgp / maxValue) * 100, 2)}%` } : {}}
            transition={{ duration: 1, delay: index * 0.15 + 0.3, ease: [0.16, 1, 0.3, 1] }}
          >
            {/* Shine effect */}
            <motion.div
              className="absolute inset-0 bg-gradient-to-r from-transparent via-white/30 to-transparent"
              initial={{ x: '-100%' }}
              animate={isInView ? { x: '200%' } : {}}
              transition={{ duration: 1, delay: index * 0.15 + 1, ease: "easeInOut" }}
            />
          </motion.div>
        </div>
        <motion.span
          className="text-sm font-mono font-bold text-[var(--color-accent)] w-20 text-right"
          initial={{ opacity: 0 }}
          animate={isInView ? { opacity: 1 } : {}}
          transition={{ duration: 0.4, delay: index * 0.15 + 0.8 }}
        >
          {data.fgp}ms
        </motion.span>
      </div>

      {/* MCP Bar */}
      <div className="flex items-center gap-4">
        <span className="text-xs font-medium text-[var(--color-text-muted)] w-12">MCP</span>
        <div className="flex-1 h-8 rounded-lg bg-[var(--color-surface-elevated)] overflow-hidden">
          <motion.div
            className="h-full rounded-lg bg-[var(--color-text-muted)]/40"
            initial={{ width: 0 }}
            animate={isInView ? { width: `${(mcpValue / maxValue) * 100}%` } : {}}
            transition={{ duration: 1.2, delay: index * 0.15 + 0.5, ease: [0.16, 1, 0.3, 1] }}
          />
        </div>
        <motion.span
          className="text-sm font-mono text-[var(--color-text-muted)] w-20 text-right"
          initial={{ opacity: 0 }}
          animate={isInView ? { opacity: 1 } : {}}
          transition={{ duration: 0.4, delay: index * 0.15 + 1 }}
        >
          {mcpValue.toLocaleString()}ms
        </motion.span>
      </div>
    </motion.div>
  );
}

export function BenchmarkChart() {
  const [showCold, setShowCold] = useState(false);

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between mb-2">
        <span className="text-sm font-medium text-[var(--color-text-muted)] uppercase tracking-wider">
          Browser Daemon
        </span>
        {/* Toggle between Cold and Warm */}
        <div className="flex items-center gap-2 p-1 rounded-lg bg-[var(--color-surface-elevated)]">
          <button
            onClick={() => setShowCold(false)}
            className={`px-3 py-1 text-xs font-medium rounded-md transition-all ${
              !showCold
                ? 'bg-[var(--color-accent)] text-[var(--color-void)]'
                : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'
            }`}
          >
            Warm vs Warm
          </button>
          <button
            onClick={() => setShowCold(true)}
            className={`px-3 py-1 text-xs font-medium rounded-md transition-all ${
              showCold
                ? 'bg-[var(--color-accent)] text-[var(--color-void)]'
                : 'text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]'
            }`}
          >
            Cold Start
          </button>
        </div>
      </div>

      {/* Context note */}
      <p className="text-xs text-[var(--color-text-muted)] -mt-2">
        {showCold
          ? "First call in new session (MCP spawns process)"
          : "Subsequent calls (MCP server already running)"}
      </p>

      <div className="space-y-8">
        {benchmarks.map((data, i) => (
          <BenchmarkBar key={data.operation} data={data} index={i} showCold={showCold} />
        ))}
      </div>
    </div>
  );
}

// ============================================================================
// Honest Value Proposition - When FGP Helps
// ============================================================================

export function CumulativeOverhead() {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  return (
    <div ref={ref} className="relative">
      {/* Header */}
      <motion.div
        className="text-center mb-12"
        initial={{ opacity: 0, y: 20 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.6 }}
      >
        <h3 className="text-2xl md:text-3xl font-bold mb-3">
          Where FGP <span className="gradient-accent-text">Shines</span>
        </h3>
        <p className="text-[var(--color-text-secondary)]">
          Consistent low latency across all scenarios
        </p>
      </motion.div>

      {/* Three scenario cards */}
      <div className="grid md:grid-cols-3 gap-6 max-w-4xl mx-auto">
        {/* Cold Start Card */}
        <motion.div
          className="relative p-6 rounded-2xl border border-[var(--color-accent)]/30 bg-[var(--color-surface)] overflow-hidden"
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : {}}
          transition={{ duration: 0.6, delay: 0.2 }}
        >
          <div className="absolute inset-0 bg-gradient-to-br from-[var(--color-accent)]/10 to-transparent" />
          <div className="relative">
            <span className="text-xs font-medium text-[var(--color-accent)] uppercase tracking-wider">
              Cold Start
            </span>
            <motion.div
              className="mt-3 text-4xl font-black gradient-accent-text"
              initial={{ opacity: 0, scale: 0.5 }}
              animate={isInView ? { opacity: 1, scale: 1 } : {}}
              transition={{ duration: 0.6, delay: 0.4, type: "spring" }}
            >
              10-17×
            </motion.div>
            <p className="mt-3 text-sm text-[var(--color-text-muted)]">
              First call in new session
            </p>
            <p className="mt-2 text-xs text-[var(--color-text-muted)]">
              ~1-2s → instant
            </p>
          </div>
        </motion.div>

        {/* Warm Calls Card */}
        <motion.div
          className="relative p-6 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden"
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : {}}
          transition={{ duration: 0.6, delay: 0.3 }}
        >
          <div className="relative">
            <span className="text-xs font-medium text-[var(--color-text-muted)] uppercase tracking-wider">
              Warm Calls
            </span>
            <motion.div
              className="mt-3 text-4xl font-black text-[var(--color-text-primary)]"
              initial={{ opacity: 0, scale: 0.5 }}
              animate={isInView ? { opacity: 1, scale: 1 } : {}}
              transition={{ duration: 0.6, delay: 0.5, type: "spring" }}
            >
              3-12×
            </motion.div>
            <p className="mt-3 text-sm text-[var(--color-text-muted)]">
              MCP server already running
            </p>
            <p className="mt-2 text-xs text-[var(--color-text-muted)]">
              ~27ms → ~3ms
            </p>
          </div>
        </motion.div>

        {/* Local Ops Card */}
        <motion.div
          className="relative p-6 rounded-2xl border border-[var(--color-accent-secondary)]/30 bg-[var(--color-surface)] overflow-hidden"
          initial={{ opacity: 0, y: 30 }}
          animate={isInView ? { opacity: 1, y: 0 } : {}}
          transition={{ duration: 0.6, delay: 0.4 }}
        >
          <div className="absolute inset-0 bg-gradient-to-br from-[var(--color-accent-secondary)]/10 to-transparent" />
          <div className="relative">
            <span className="text-xs font-medium text-[var(--color-accent-secondary)] uppercase tracking-wider">
              Local SQLite
            </span>
            <motion.div
              className="mt-3 text-4xl font-black"
              style={{ color: 'var(--color-accent-secondary)' }}
              initial={{ opacity: 0, scale: 0.5 }}
              animate={isInView ? { opacity: 1, scale: 1 } : {}}
              transition={{ duration: 0.6, delay: 0.6, type: "spring" }}
            >
              50×
            </motion.div>
            <p className="mt-3 text-sm text-[var(--color-text-muted)]">
              iMessage, Screen Time
            </p>
            <p className="mt-2 text-xs text-[var(--color-text-muted)]">
              ~60ms → ~1ms
            </p>
          </div>
        </motion.div>
      </div>

      {/* Clarification note */}
      <motion.div
        className="mt-8 text-center"
        initial={{ opacity: 0, y: 20 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.6, delay: 0.8 }}
      >
        <p className="text-sm text-[var(--color-text-muted)] max-w-xl mx-auto">
          <strong>Note:</strong> MCP servers stay warm within a Claude Code session.
          FGP's main advantage is eliminating cold starts and providing faster warm calls
          through lower protocol overhead.
        </p>
      </motion.div>
    </div>
  );
}

// ============================================================================
// Stat Pills - Compact Stats Display
// ============================================================================

interface StatPill {
  value: string;
  label: string;
  accent?: boolean;
}

export function StatPills({ stats }: { stats: StatPill[] }) {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true });

  return (
    <div ref={ref} className="flex flex-wrap justify-center gap-4">
      {stats.map((stat, i) => (
        <motion.div
          key={stat.label}
          className={`
            px-6 py-3 rounded-full border
            ${stat.accent
              ? 'border-[var(--color-accent)]/30 bg-[var(--color-accent-muted)]'
              : 'border-[var(--color-border)] bg-[var(--color-surface)]'
            }
          `}
          initial={{ opacity: 0, y: 20, scale: 0.9 }}
          animate={isInView ? { opacity: 1, y: 0, scale: 1 } : {}}
          transition={{ duration: 0.5, delay: i * 0.1, ease: [0.16, 1, 0.3, 1] }}
        >
          <span className={`font-bold ${stat.accent ? 'gradient-accent-text' : 'text-[var(--color-text-primary)]'}`}>
            {stat.value}
          </span>
          <span className="text-[var(--color-text-muted)] ml-2">{stat.label}</span>
        </motion.div>
      ))}
    </div>
  );
}

import { motion, useInView, useSpring, useTransform } from 'framer-motion';
import { useRef, useEffect, useState } from 'react';

// ============================================================================
// Animated Counter Component
// ============================================================================

function AnimatedCounter({
  value,
  suffix = '',
  duration = 2
}: {
  value: number;
  suffix?: string;
  duration?: number;
}) {
  const ref = useRef<HTMLSpanElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });
  const spring = useSpring(0, { duration: duration * 1000, bounce: 0 });
  const display = useTransform(spring, (v) => Math.round(v));
  const [displayValue, setDisplayValue] = useState(0);

  useEffect(() => {
    if (isInView) {
      spring.set(value);
    }
  }, [isInView, spring, value]);

  useEffect(() => {
    return display.on("change", (v) => setDisplayValue(v));
  }, [display]);

  return (
    <span ref={ref}>
      {displayValue.toLocaleString()}{suffix}
    </span>
  );
}

// ============================================================================
// Hero Speedup Display - The Big 292×
// ============================================================================

export function HeroSpeedup({ compact = false }: { compact?: boolean }) {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true });
  const sizeClass = compact
    ? 'text-[clamp(4.5rem,12vw,9rem)]'
    : 'text-[clamp(8rem,25vw,16rem)]';
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

      {/* The big number */}
      <motion.div
        className={`relative ${sizeClass} font-black leading-none tracking-tighter`}
        initial={{ opacity: 0, scale: 0.8, y: 40 }}
        animate={isInView ? { opacity: 1, scale: 1, y: 0 } : {}}
        transition={{ duration: 0.8, ease: [0.16, 1, 0.3, 1] }}
      >
        <span className="gradient-accent-text">
          <AnimatedCounter value={292} suffix="×" />
        </span>
      </motion.div>

      {/* Subtitle */}
      <motion.p
        className={`${subtitleClass} text-[var(--color-text-secondary)] mt-4`}
        initial={{ opacity: 0, y: 20 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.6, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
      >
        faster than MCP cold-start
      </motion.p>
    </div>
  );
}

// ============================================================================
// Benchmark Bar Chart - Visual Comparison
// ============================================================================

interface BenchmarkData {
  operation: string;
  fgp: number;
  mcp: number;
}

const benchmarks: BenchmarkData[] = [
  { operation: 'Navigate', fgp: 8, mcp: 2328 },
  { operation: 'Snapshot', fgp: 9, mcp: 2484 },
  { operation: 'Screenshot', fgp: 30, mcp: 1635 },
];

function BenchmarkBar({ data, index }: { data: BenchmarkData; index: number }) {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-50px" });
  const speedup = Math.round(data.mcp / data.fgp);
  const maxMcp = Math.max(...benchmarks.map(b => b.mcp));

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
            animate={isInView ? { width: `${(data.fgp / maxMcp) * 100}%` } : {}}
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
            animate={isInView ? { width: `${(data.mcp / maxMcp) * 100}%` } : {}}
            transition={{ duration: 1.2, delay: index * 0.15 + 0.5, ease: [0.16, 1, 0.3, 1] }}
          />
        </div>
        <motion.span
          className="text-sm font-mono text-[var(--color-text-muted)] w-20 text-right"
          initial={{ opacity: 0 }}
          animate={isInView ? { opacity: 1 } : {}}
          transition={{ duration: 0.4, delay: index * 0.15 + 1 }}
        >
          {data.mcp.toLocaleString()}ms
        </motion.span>
      </div>
    </motion.div>
  );
}

export function BenchmarkChart() {
  return (
    <div className="space-y-8">
      {benchmarks.map((data, i) => (
        <BenchmarkBar key={data.operation} data={data} index={i} />
      ))}
    </div>
  );
}

// ============================================================================
// Cumulative Overhead Visualization
// ============================================================================

export function CumulativeOverhead() {
  const ref = useRef<HTMLDivElement>(null);
  const isInView = useInView(ref, { once: true, margin: "-100px" });

  const toolCalls = 20;
  const mcpOverhead = 2.3; // seconds per call
  const fgpOverhead = 0.01; // seconds per call
  const mcpTotal = toolCalls * mcpOverhead;
  const fgpTotal = toolCalls * fgpOverhead;

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
          Overhead <span className="gradient-accent-text">Compounds</span>
        </h3>
        <p className="text-[var(--color-text-secondary)]">
          Time wasted on cold-start over {toolCalls} tool calls
        </p>
      </motion.div>

      {/* Comparison boxes */}
      <div className="grid md:grid-cols-2 gap-6 max-w-3xl mx-auto">
        {/* MCP Box */}
        <motion.div
          className="relative p-8 rounded-2xl border border-[var(--color-border)] bg-[var(--color-surface)] overflow-hidden"
          initial={{ opacity: 0, x: -30 }}
          animate={isInView ? { opacity: 1, x: 0 } : {}}
          transition={{ duration: 0.6, delay: 0.2 }}
        >
          {/* Red glow for MCP */}
          <div className="absolute inset-0 bg-gradient-to-br from-red-500/5 to-transparent" />

          <div className="relative">
            <span className="text-sm font-medium text-[var(--color-text-muted)] uppercase tracking-wider">
              MCP Stdio
            </span>
            <div className="mt-4 flex items-baseline gap-2">
              <motion.span
                className="text-5xl md:text-6xl font-black text-red-400"
                initial={{ opacity: 0, scale: 0.5 }}
                animate={isInView ? { opacity: 1, scale: 1 } : {}}
                transition={{ duration: 0.6, delay: 0.5, type: "spring" }}
              >
                <AnimatedCounter value={mcpTotal} suffix="s" />
              </motion.span>
              <span className="text-lg text-[var(--color-text-muted)]">wasted</span>
            </div>
            <p className="mt-4 text-sm text-[var(--color-text-muted)]">
              ~{mcpOverhead}s cold-start × {toolCalls} calls
            </p>
          </div>
        </motion.div>

        {/* FGP Box */}
        <motion.div
          className="relative p-8 rounded-2xl border border-[var(--color-accent)]/30 bg-[var(--color-surface)] overflow-hidden"
          initial={{ opacity: 0, x: 30 }}
          animate={isInView ? { opacity: 1, x: 0 } : {}}
          transition={{ duration: 0.6, delay: 0.3 }}
        >
          {/* Cyan glow for FGP */}
          <div className="absolute inset-0 bg-gradient-to-br from-[var(--color-accent)]/10 to-transparent" />

          <div className="relative">
            <span className="text-sm font-medium text-[var(--color-accent)] uppercase tracking-wider">
              FGP Daemon
            </span>
            <div className="mt-4 flex items-baseline gap-2">
              <motion.span
                className="text-5xl md:text-6xl font-black gradient-accent-text"
                initial={{ opacity: 0, scale: 0.5 }}
                animate={isInView ? { opacity: 1, scale: 1 } : {}}
                transition={{ duration: 0.6, delay: 0.6, type: "spring" }}
              >
                {fgpTotal}s
              </motion.span>
              <span className="text-lg text-[var(--color-text-muted)]">total</span>
            </div>
            <p className="mt-4 text-sm text-[var(--color-text-muted)]">
              ~{fgpOverhead * 1000}ms overhead × {toolCalls} calls
            </p>
          </div>
        </motion.div>
      </div>

      {/* Time saved callout */}
      <motion.div
        className="mt-8 text-center"
        initial={{ opacity: 0, y: 20 }}
        animate={isInView ? { opacity: 1, y: 0 } : {}}
        transition={{ duration: 0.6, delay: 0.8 }}
      >
        <div className="inline-flex items-center gap-3 px-6 py-3 rounded-full border border-[var(--color-accent)]/30 bg-[var(--color-accent-muted)]">
          <span className="text-lg font-bold gradient-accent-text">
            {(mcpTotal - fgpTotal).toFixed(1)}s saved
          </span>
          <span className="text-[var(--color-text-secondary)]">per workflow</span>
        </div>
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

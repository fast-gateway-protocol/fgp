import { createFileRoute, Link } from '@tanstack/react-router';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { Copy, Check, Terminal, Zap, BookOpen, Code, ArrowRight, Package } from 'lucide-react';

function CopyCommand({ command }: { command: string }) {
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

function DocsPage() {
  return (
    <div className="section">
      <div className="container max-w-4xl">
        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <h1 className="text-section mb-4">Documentation</h1>
          <p className="text-[var(--color-text-secondary)] text-lg mb-12">
            Get started with FGP in minutes. Install the CLI, add packages, and start using fast
            daemons with your AI agents.
          </p>
        </motion.div>

        {/* Quick Start */}
        <motion.section
          className="mb-16"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <Terminal className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">Quick Start</h2>
          </div>

          <div className="space-y-8">
            <div>
              <h3 className="font-medium mb-3">1. Install the FGP CLI</h3>
              <CopyCommand command="curl -sSL getfgp.com/install | bash" />
              <p className="text-sm text-[var(--color-text-muted)] mt-2">
                This installs the <code className="text-[var(--color-accent)]">fgp</code> command
                globally.
              </p>
            </div>

            <div>
              <h3 className="font-medium mb-3">2. Install a package</h3>
              <CopyCommand command="fgp install browser" />
              <p className="text-sm text-[var(--color-text-muted)] mt-2">
                Downloads and installs the browser daemon to{' '}
                <code className="text-[var(--color-accent)]">~/.fgp/services/browser/</code>
              </p>
            </div>

            <div>
              <h3 className="font-medium mb-3">3. Start the daemon</h3>
              <CopyCommand command="fgp start browser" />
              <p className="text-sm text-[var(--color-text-muted)] mt-2">
                Starts the daemon in the background, listening on a UNIX socket.
              </p>
            </div>

            <div>
              <h3 className="font-medium mb-3">4. Use it</h3>
              <CopyCommand command='fgp call browser.open --url "https://example.com"' />
              <p className="text-sm text-[var(--color-text-muted)] mt-2">
                Call any method on the daemon. Results are returned as JSON.
              </p>
            </div>
          </div>
        </motion.section>

        {/* How It Works */}
        <motion.section
          className="mb-16"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <Zap className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">How FGP Works</h2>
          </div>

          <div className="card">
            <div className="prose prose-invert max-w-none">
              <p className="text-[var(--color-text-secondary)]">
                FGP (Fast Gateway Protocol) replaces slow MCP stdio servers with persistent UNIX
                socket daemons. Instead of spawning a new process for each tool call (~2.3s
                overhead), FGP keeps daemons warm and ready (~10-50ms latency).
              </p>

              <div className="grid md:grid-cols-2 gap-6 mt-6">
                <div className="p-4 rounded-lg bg-[var(--color-surface-elevated)]">
                  <h4 className="font-medium text-[var(--color-text-muted)] mb-2">MCP (Slow)</h4>
                  <ul className="text-sm text-[var(--color-text-secondary)] space-y-1">
                    <li>• Spawn new process per call</li>
                    <li>• Load runtime + dependencies</li>
                    <li>• Initialize connections</li>
                    <li>• Execute tool</li>
                    <li>• ~2,300ms per call</li>
                  </ul>
                </div>
                <div className="p-4 rounded-lg bg-[var(--color-accent-muted)]">
                  <h4 className="font-medium text-[var(--color-accent)] mb-2">FGP (Fast)</h4>
                  <ul className="text-sm text-[var(--color-text-secondary)] space-y-1">
                    <li>• Daemon already running</li>
                    <li>• Connections pre-established</li>
                    <li>• Direct socket call</li>
                    <li>• Execute tool</li>
                    <li>• ~8-50ms per call</li>
                  </ul>
                </div>
              </div>
            </div>
          </div>
        </motion.section>

        {/* Protocol */}
        <motion.section
          className="mb-16"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <Code className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">Protocol</h2>
          </div>

          <div className="card">
            <p className="text-[var(--color-text-secondary)] mb-4">
              FGP uses NDJSON (newline-delimited JSON) over UNIX sockets. Each daemon listens at:
            </p>
            <code className="block bg-[var(--color-surface-elevated)] p-3 rounded text-sm mb-6">
              ~/.fgp/services/{'<name>'}/daemon.sock
            </code>

            <h4 className="font-medium mb-3">Request Format</h4>
            <pre className="bg-[var(--color-surface-elevated)] p-4 rounded text-sm overflow-x-auto mb-6">
              {`{
  "id": "uuid",
  "v": 1,
  "method": "service.action",
  "params": { ... }
}`}
            </pre>

            <h4 className="font-medium mb-3">Response Format</h4>
            <pre className="bg-[var(--color-surface-elevated)] p-4 rounded text-sm overflow-x-auto">
              {`{
  "id": "uuid",
  "ok": true,
  "result": { ... },
  "meta": {
    "server_ms": 12.5,
    "protocol_v": 1
  }
}`}
            </pre>
          </div>
        </motion.section>

        {/* AI Agent Integration */}
        <motion.section
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <BookOpen className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">AI Agent Integration</h2>
          </div>

          <div className="card">
            <p className="text-[var(--color-text-secondary)] mb-4">
              FGP packages include skills for popular AI coding agents. When you install a package,
              skills are automatically installed for detected agents.
            </p>

            <div className="grid md:grid-cols-2 gap-4">
              {[
                { name: 'Claude Code', path: '~/.claude/skills/' },
                { name: 'Cursor', path: '~/.cursor/rules/' },
                { name: 'Windsurf', path: '~/.windsurf/skills/' },
                { name: 'Continue', path: '~/.continue/skills/' },
              ].map((agent) => (
                <div
                  key={agent.name}
                  className="p-4 rounded-lg bg-[var(--color-surface-elevated)]"
                >
                  <h4 className="font-medium">{agent.name}</h4>
                  <code className="text-xs text-[var(--color-text-muted)]">{agent.path}</code>
                </div>
              ))}
            </div>
          </div>
        </motion.section>

        {/* Publishing */}
        <motion.section
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.5, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-secondary-muted)] text-[var(--color-accent-secondary)]">
              <Package className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">Create Your Own Package</h2>
          </div>

          <Link
            to="/docs/publishing"
            className="card group flex items-center justify-between hover:border-[var(--color-accent)]/30"
          >
            <div>
              <h3 className="font-semibold group-hover:text-[var(--color-accent)] transition-colors">
                Publishing Guide
              </h3>
              <p className="text-sm text-[var(--color-text-muted)] mt-1">
                Learn how to create and publish your own FGP daemon package to the marketplace.
              </p>
            </div>
            <ArrowRight className="w-5 h-5 text-[var(--color-text-muted)] group-hover:text-[var(--color-accent)] transition-colors" />
          </Link>
        </motion.section>
      </div>
    </div>
  );
}

export const Route = createFileRoute('/docs/')({
  component: DocsPage,
});

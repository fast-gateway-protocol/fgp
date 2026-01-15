import { createFileRoute, Link } from '@tanstack/react-router';
import { useState } from 'react';
import { motion } from 'framer-motion';
import { Copy, Check, Package, FileCode, GitBranch, CheckCircle, ArrowLeft } from 'lucide-react';

function CopyCommand({ command }: { command: string }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(command);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div className="code-block flex items-center justify-between gap-4">
      <code className="text-sm">
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

function PublishingPage() {
  return (
    <div className="section">
      <div className="container max-w-4xl">
        <Link
          to="/docs"
          className="inline-flex items-center gap-2 text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)] transition-colors mb-8"
        >
          <ArrowLeft className="w-4 h-4" />
          Back to Docs
        </Link>

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <h1 className="text-section mb-4">Publishing a Package</h1>
          <p className="text-[var(--color-text-secondary)] text-lg mb-12">
            Create and publish your own FGP daemon package to the marketplace.
          </p>
        </motion.div>

        {/* Package Structure */}
        <motion.section
          className="mb-16"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <Package className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">Package Structure</h2>
          </div>

          <div className="card">
            <p className="text-[var(--color-text-secondary)] mb-4">
              Every FGP package must follow this directory structure:
            </p>
            <pre className="bg-[var(--color-surface-elevated)] p-4 rounded text-sm overflow-x-auto mb-6">
{`my-daemon/
├── manifest.json      # Package metadata (required)
├── bin/
│   └── my-daemon      # Compiled daemon binary
├── skills/            # AI agent skill files (optional)
│   ├── claude-code/
│   │   └── my-daemon.md
│   └── cursor/
│       └── my-daemon.md
└── README.md          # Documentation`}
            </pre>
            <p className="text-sm text-[var(--color-text-muted)]">
              The daemon binary should be a self-contained executable that listens on a UNIX socket.
            </p>
          </div>
        </motion.section>

        {/* manifest.json */}
        <motion.section
          className="mb-16"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <FileCode className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">manifest.json Schema</h2>
          </div>

          <div className="card">
            <p className="text-[var(--color-text-secondary)] mb-4">
              The manifest file describes your package and its capabilities:
            </p>
            <pre className="bg-[var(--color-surface-elevated)] p-4 rounded text-sm overflow-x-auto">
{`{
  "name": "my-daemon",
  "version": "1.0.0",
  "description": "Description of what this daemon does",
  "author": "Your Name",
  "license": "MIT",
  "repository": "https://github.com/you/my-daemon",
  "platforms": ["darwin", "linux"],
  "categories": ["productivity"],
  "auth": {
    "type": "oauth2",          // oauth2 | bearer_token | cli | none | stateful
    "provider": "google",      // OAuth provider name
    "scopes": ["scope1", "scope2"],
    "setup": "Run 'fgp auth my-daemon' to authenticate"
  },
  "methods": [
    {
      "name": "my-daemon.action",
      "description": "Performs an action",
      "params": [
        {
          "name": "input",
          "type": "string",
          "required": true
        }
      ]
    }
  ],
  "benchmark": {
    "avg_latency_ms": 15,
    "vs_mcp_speedup": "100x"
  }
}`}
            </pre>
          </div>
        </motion.section>

        {/* Testing */}
        <motion.section
          className="mb-16"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.3, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <CheckCircle className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">Testing Requirements</h2>
          </div>

          <div className="card">
            <p className="text-[var(--color-text-secondary)] mb-4">
              Before publishing, ensure your daemon passes these checks:
            </p>
            <ul className="space-y-3 text-[var(--color-text-secondary)]">
              <li className="flex items-start gap-2">
                <Check className="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" />
                <span><strong className="text-[var(--color-text-primary)]">Health check:</strong> Daemon responds to <code className="text-[var(--color-accent)]">health</code> method</span>
              </li>
              <li className="flex items-start gap-2">
                <Check className="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" />
                <span><strong className="text-[var(--color-text-primary)]">Methods list:</strong> Daemon responds to <code className="text-[var(--color-accent)]">methods</code> with all available methods</span>
              </li>
              <li className="flex items-start gap-2">
                <Check className="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" />
                <span><strong className="text-[var(--color-text-primary)]">Graceful shutdown:</strong> Daemon responds to <code className="text-[var(--color-accent)]">stop</code> and exits cleanly</span>
              </li>
              <li className="flex items-start gap-2">
                <Check className="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" />
                <span><strong className="text-[var(--color-text-primary)]">Socket cleanup:</strong> Socket file is removed on stop</span>
              </li>
              <li className="flex items-start gap-2">
                <Check className="w-5 h-5 text-[var(--color-success)] flex-shrink-0 mt-0.5" />
                <span><strong className="text-[var(--color-text-primary)]">Error handling:</strong> Invalid methods return proper error responses</span>
              </li>
            </ul>

            <div className="mt-6 pt-6 border-t border-[var(--color-border)]">
              <h4 className="font-medium mb-3">Test your daemon locally</h4>
              <CopyCommand command="fgp test ./my-daemon" />
            </div>
          </div>
        </motion.section>

        {/* Submitting */}
        <motion.section
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, delay: 0.4, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="flex items-center gap-3 mb-6">
            <div className="p-2 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
              <GitBranch className="w-5 h-5" />
            </div>
            <h2 className="text-xl font-semibold">Submitting to the Marketplace</h2>
          </div>

          <div className="card">
            <ol className="space-y-6 text-[var(--color-text-secondary)]">
              <li>
                <h4 className="font-medium text-[var(--color-text-primary)] mb-2">1. Create a GitHub release</h4>
                <p className="mb-3">Tag your release with the version number and upload platform-specific tarballs:</p>
                <ul className="text-sm space-y-1 ml-4">
                  <li>• <code className="text-[var(--color-accent)]">my-daemon-1.0.0-darwin-arm64.tar.gz</code></li>
                  <li>• <code className="text-[var(--color-accent)]">my-daemon-1.0.0-darwin-x64.tar.gz</code></li>
                  <li>• <code className="text-[var(--color-accent)]">my-daemon-1.0.0-linux-x64.tar.gz</code></li>
                </ul>
              </li>
              <li>
                <h4 className="font-medium text-[var(--color-text-primary)] mb-2">2. Submit a PR to the registry</h4>
                <p className="mb-3">Add your package to the FGP registry repository:</p>
                <CopyCommand command="gh repo fork fast-gateway-protocol/registry" />
              </li>
              <li>
                <h4 className="font-medium text-[var(--color-text-primary)] mb-2">3. Add your package entry</h4>
                <p>Add your package to <code className="text-[var(--color-accent)]">packages/</code> with a JSON file containing your manifest.</p>
              </li>
              <li>
                <h4 className="font-medium text-[var(--color-text-primary)] mb-2">4. Wait for review</h4>
                <p>The FGP team will review your submission and run automated tests. Once approved, your package will appear in the marketplace.</p>
              </li>
            </ol>

            <div className="mt-6 pt-6 border-t border-[var(--color-border)] text-sm text-[var(--color-text-muted)]">
              <p>
                <strong className="text-[var(--color-text-secondary)]">Note:</strong> Verified badges are granted to packages that pass additional security review and meet quality standards.
              </p>
            </div>
          </div>
        </motion.section>
      </div>
    </div>
  );
}

export const Route = createFileRoute('/docs/publishing')({
  component: PublishingPage,
});

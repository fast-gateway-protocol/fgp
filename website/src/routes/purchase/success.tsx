import { createFileRoute, Link, useSearch } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import { CheckCircle, Copy, Check, Terminal, ArrowRight } from 'lucide-react';
import { useState, useEffect } from 'react';

interface SearchParams {
  session_id?: string;
}

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

function PurchaseSuccessPage() {
  const { session_id } = useSearch({ from: '/purchase/success' }) as SearchParams;
  const [licenseKey, setLicenseKey] = useState<string | null>(null);
  const [skillName, setSkillName] = useState<string | null>(null);
  const [copied, setCopied] = useState(false);

  // In production, fetch purchase details from the API using session_id
  useEffect(() => {
    // Simulated for now - would fetch from API
    setLicenseKey('sk_live_demo_xxxxxxxxxxxxx');
    setSkillName('twitter-research');
  }, [session_id]);

  const copyLicenseKey = async () => {
    if (licenseKey) {
      await navigator.clipboard.writeText(licenseKey);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  return (
    <div className="section min-h-[80vh] flex items-center">
      <div className="container max-w-2xl">
        <motion.div
          className="text-center"
          initial={{ opacity: 0, scale: 0.95 }}
          animate={{ opacity: 1, scale: 1 }}
          transition={{ duration: 0.5, ease: [0.16, 1, 0.3, 1] }}
        >
          <div className="inline-flex items-center justify-center w-20 h-20 rounded-full bg-[var(--color-success)]/20 mb-6">
            <CheckCircle className="w-10 h-10 text-[var(--color-success)]" />
          </div>
          <h1 className="text-3xl font-bold mb-2">Purchase Complete!</h1>
          <p className="text-[var(--color-text-secondary)] mb-8">
            Thank you for your purchase. Your license key is ready.
          </p>
        </motion.div>

        <motion.div
          className="card"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.1, ease: [0.16, 1, 0.3, 1] }}
        >
          {/* License Key */}
          <div className="mb-6">
            <h2 className="font-semibold mb-3 flex items-center gap-2">
              <Terminal className="w-5 h-5 text-[var(--color-accent)]" />
              Your License Key
            </h2>
            <div className="flex items-center gap-2">
              <code className="flex-1 px-4 py-3 bg-[var(--color-surface-elevated)] rounded-lg text-sm font-mono overflow-x-auto">
                {licenseKey || 'Loading...'}
              </code>
              <button
                onClick={copyLicenseKey}
                disabled={!licenseKey}
                className="p-3 rounded-lg bg-[var(--color-surface-elevated)] hover:bg-[var(--color-surface-hover)] transition-colors"
              >
                {copied ? (
                  <Check className="w-5 h-5 text-[var(--color-success)]" />
                ) : (
                  <Copy className="w-5 h-5 text-[var(--color-text-muted)]" />
                )}
              </button>
            </div>
            <p className="text-xs text-[var(--color-text-muted)] mt-2">
              Save this key securely. You'll need it to install and activate the skill.
            </p>
          </div>

          {/* Install Instructions */}
          <div className="pt-6 border-t border-[var(--color-border)]">
            <h2 className="font-semibold mb-4">Install Your Skill</h2>
            <div className="space-y-3">
              <InstallCommand command={`fgp skill install ${skillName || 'skill-name'} --license ${licenseKey || 'YOUR_LICENSE_KEY'}`} />
            </div>
          </div>

          {/* Next Steps */}
          <div className="mt-6 pt-6 border-t border-[var(--color-border)]">
            <h2 className="font-semibold mb-3">What's Next?</h2>
            <ul className="space-y-2 text-sm text-[var(--color-text-secondary)]">
              <li className="flex items-center gap-2">
                <CheckCircle className="w-4 h-4 text-[var(--color-success)]" />
                A confirmation email has been sent to your inbox
              </li>
              <li className="flex items-center gap-2">
                <CheckCircle className="w-4 h-4 text-[var(--color-success)]" />
                Your license is valid for up to 3 machines
              </li>
              <li className="flex items-center gap-2">
                <CheckCircle className="w-4 h-4 text-[var(--color-success)]" />
                7-day money-back guarantee if you're not satisfied
              </li>
            </ul>
          </div>
        </motion.div>

        <motion.div
          className="mt-8 flex flex-col sm:flex-row gap-4 justify-center"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.5, delay: 0.2, ease: [0.16, 1, 0.3, 1] }}
        >
          <Link to="/dashboard" className="btn btn-primary flex items-center gap-2">
            View My Purchases
            <ArrowRight className="w-4 h-4" />
          </Link>
          <Link to="/marketplace" className="btn btn-secondary">
            Back to Marketplace
          </Link>
        </motion.div>
      </div>
    </div>
  );
}

export const Route = createFileRoute('/purchase/success')({
  component: PurchaseSuccessPage,
});

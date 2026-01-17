import { createFileRoute, Link, useNavigate } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import { Package, Key, Copy, Check, ExternalLink, Calendar, DollarSign, Loader2 } from 'lucide-react';
import { useState, useEffect } from 'react';
import { useAuth } from '@/context/AuthContext';
import { fetchPurchases, formatPrice, type Purchase } from '@/lib/api';

function CopyButton({ text }: { text: string }) {
  const [copied, setCopied] = useState(false);

  const copy = async () => {
    await navigator.clipboard.writeText(text);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <button
      onClick={copy}
      className="p-1.5 rounded hover:bg-[var(--color-surface-hover)] transition-colors"
      title="Copy license key"
    >
      {copied ? (
        <Check className="w-4 h-4 text-[var(--color-success)]" />
      ) : (
        <Copy className="w-4 h-4 text-[var(--color-text-muted)]" />
      )}
    </button>
  );
}

function PurchaseCard({ purchase }: { purchase: Purchase }) {
  const statusColors = {
    active: 'var(--color-success)',
    refunded: 'var(--color-text-muted)',
    revoked: 'var(--color-error)',
  };

  const statusColor = statusColors[purchase.license_status] || 'var(--color-text-muted)';
  const formattedDate = new Date(purchase.purchased_at).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  });

  return (
    <motion.div
      className="card"
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
    >
      <div className="flex items-start justify-between gap-4">
        <div className="flex items-start gap-4">
          <div className="p-3 rounded-lg bg-[var(--color-accent-muted)] text-[var(--color-accent)]">
            <Package className="w-6 h-6" />
          </div>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold">{purchase.skill.name}</h3>
              <span
                className="text-xs px-2 py-0.5 rounded-full"
                style={{
                  color: statusColor,
                  backgroundColor: `color-mix(in srgb, ${statusColor} 15%, transparent)`,
                }}
              >
                {purchase.license_status}
              </span>
            </div>
            <p className="text-sm text-[var(--color-text-muted)] mt-1">
              {purchase.skill.description}
            </p>
            <div className="flex items-center gap-4 mt-3 text-xs text-[var(--color-text-muted)]">
              <span className="flex items-center gap-1">
                <Calendar className="w-3 h-3" />
                {formattedDate}
              </span>
              <span className="flex items-center gap-1">
                <DollarSign className="w-3 h-3" />
                {formatPrice(purchase.amount_cents)}
              </span>
            </div>
          </div>
        </div>
        <Link
          to="/marketplace/$packageName"
          params={{ packageName: purchase.skill.slug }}
          className="text-sm text-[var(--color-accent)] hover:underline flex items-center gap-1"
        >
          View
          <ExternalLink className="w-3 h-3" />
        </Link>
      </div>

      {/* License Key */}
      {purchase.license_status === 'active' && (
        <div className="mt-4 pt-4 border-t border-[var(--color-border)]">
          <div className="flex items-center gap-2 text-sm">
            <Key className="w-4 h-4 text-[var(--color-text-muted)]" />
            <code className="flex-1 px-2 py-1 bg-[var(--color-surface-elevated)] rounded text-xs font-mono truncate">
              {purchase.license_key}
            </code>
            <CopyButton text={purchase.license_key} />
          </div>
          <p className="text-xs text-[var(--color-text-muted)] mt-2">
            Install: <code className="text-[var(--color-accent)]">fgp skill install {purchase.skill.slug} --license {purchase.license_key}</code>
          </p>
        </div>
      )}
    </motion.div>
  );
}

function DashboardPage() {
  const { isAuthenticated, isLoading: authLoading } = useAuth();
  const navigate = useNavigate();
  const [purchases, setPurchases] = useState<Purchase[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      navigate({ to: '/marketplace' });
      return;
    }

    if (isAuthenticated) {
      loadPurchases();
    }
  }, [isAuthenticated, authLoading, navigate]);

  const loadPurchases = async () => {
    try {
      const data = await fetchPurchases();
      setPurchases(data);
    } catch (err) {
      setError('Failed to load purchases');
    } finally {
      setIsLoading(false);
    }
  };

  if (authLoading || (!isAuthenticated && !authLoading)) {
    return (
      <div className="section min-h-[60vh] flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-[var(--color-accent)]" />
      </div>
    );
  }

  return (
    <div className="section">
      <div className="container max-w-4xl">
        <motion.div
          className="mb-8"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ duration: 0.6, ease: [0.16, 1, 0.3, 1] }}
        >
          <h1 className="text-section">My Purchases</h1>
          <p className="text-[var(--color-text-secondary)] mt-2">
            Manage your purchased skills and license keys
          </p>
        </motion.div>

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="w-8 h-8 animate-spin text-[var(--color-accent)]" />
          </div>
        ) : error ? (
          <div className="card text-center py-12">
            <p className="text-[var(--color-error)]">{error}</p>
            <button onClick={loadPurchases} className="btn btn-secondary mt-4">
              Try Again
            </button>
          </div>
        ) : purchases.length === 0 ? (
          <motion.div
            className="card text-center py-12"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
          >
            <Package className="w-12 h-12 mx-auto text-[var(--color-text-muted)] mb-4" />
            <h2 className="text-xl font-semibold mb-2">No purchases yet</h2>
            <p className="text-[var(--color-text-muted)] mb-6">
              Browse our marketplace to find premium skills for your AI agents
            </p>
            <Link to="/marketplace" className="btn btn-primary">
              Explore Marketplace
            </Link>
          </motion.div>
        ) : (
          <div className="space-y-4">
            {purchases.map((purchase, i) => (
              <motion.div
                key={purchase.id}
                initial={{ opacity: 0, y: 20 }}
                animate={{ opacity: 1, y: 0 }}
                transition={{ delay: i * 0.05 }}
              >
                <PurchaseCard purchase={purchase} />
              </motion.div>
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export const Route = createFileRoute('/dashboard/')({
  component: DashboardPage,
});

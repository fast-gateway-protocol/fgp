import { createFileRoute, Link, useNavigate } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import {
  Package,
  DollarSign,
  Download,
  TrendingUp,
  Plus,
  ExternalLink,
  Loader2,
  AlertCircle,
  Clock,
  CheckCircle,
  XCircle,
  Star,
} from 'lucide-react';
import { useState, useEffect } from 'react';
import { useAuth } from '@/context/AuthContext';
import {
  fetchSellerStats,
  fetchSellerSkills,
  fetchSellerSales,
  becomeSeller,
  formatPrice,
  getTierColor,
  type SellerStats,
  type SellerSkill,
  type Sale,
} from '@/lib/api';

function StatCard({
  label,
  value,
  icon: Icon,
  color = 'var(--color-accent)',
}: {
  label: string;
  value: string | number;
  icon: React.ElementType;
  color?: string;
}) {
  return (
    <div className="card">
      <div className="flex items-center gap-3">
        <div
          className="p-3 rounded-lg"
          style={{ backgroundColor: `color-mix(in srgb, ${color} 15%, transparent)` }}
        >
          <Icon className="w-5 h-5" style={{ color }} />
        </div>
        <div>
          <p className="text-sm text-[var(--color-text-muted)]">{label}</p>
          <p className="text-xl font-semibold">{value}</p>
        </div>
      </div>
    </div>
  );
}

function SkillStatusBadge({ status }: { status: SellerSkill['status'] }) {
  const config = {
    active: { color: 'var(--color-success)', icon: CheckCircle, label: 'Active' },
    pending: { color: 'var(--color-accent-secondary)', icon: Clock, label: 'Pending Review' },
    rejected: { color: 'var(--color-error)', icon: XCircle, label: 'Rejected' },
    archived: { color: 'var(--color-text-muted)', icon: Package, label: 'Archived' },
  };

  const { color, icon: Icon, label } = config[status];

  return (
    <span
      className="inline-flex items-center gap-1 text-xs px-2 py-0.5 rounded-full"
      style={{
        color,
        backgroundColor: `color-mix(in srgb, ${color} 15%, transparent)`,
      }}
    >
      <Icon className="w-3 h-3" />
      {label}
    </span>
  );
}

function SkillCard({ skill }: { skill: SellerSkill }) {
  const tierColor = getTierColor(skill.tier);

  return (
    <div className="card">
      <div className="flex items-start justify-between gap-4">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2 flex-wrap">
            <h3 className="font-semibold truncate">{skill.name}</h3>
            <SkillStatusBadge status={skill.status} />
            <span
              className="text-xs px-2 py-0.5 rounded-full"
              style={{
                color: tierColor,
                backgroundColor: `color-mix(in srgb, ${tierColor} 15%, transparent)`,
              }}
            >
              {formatPrice(skill.price_cents)}
            </span>
          </div>
          <p className="text-sm text-[var(--color-text-muted)] mt-1 line-clamp-2">
            {skill.description}
          </p>
          <div className="flex items-center gap-4 mt-3 text-xs text-[var(--color-text-muted)]">
            <span className="flex items-center gap-1">
              <Download className="w-3 h-3" />
              {skill.download_count} downloads
            </span>
            {skill.rating_average && (
              <span className="flex items-center gap-1">
                <Star className="w-3 h-3 fill-[var(--color-accent-secondary)] text-[var(--color-accent-secondary)]" />
                {skill.rating_average.toFixed(1)}
              </span>
            )}
            <span className="flex items-center gap-1">
              <DollarSign className="w-3 h-3" />
              {formatPrice(skill.total_sales_cents)} earned
            </span>
          </div>
        </div>
        <Link
          to="/marketplace/$packageName"
          params={{ packageName: skill.slug }}
          className="text-sm text-[var(--color-accent)] hover:underline flex items-center gap-1 flex-shrink-0"
        >
          View
          <ExternalLink className="w-3 h-3" />
        </Link>
      </div>
    </div>
  );
}

function SaleRow({ sale }: { sale: Sale }) {
  const formattedDate = new Date(sale.purchased_at).toLocaleDateString('en-US', {
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });

  return (
    <tr className="border-b border-[var(--color-border)] last:border-0">
      <td className="py-3 pr-4">
        <Link
          to="/marketplace/$packageName"
          params={{ packageName: sale.skill_slug }}
          className="font-medium hover:text-[var(--color-accent)]"
        >
          {sale.skill_name}
        </Link>
      </td>
      <td className="py-3 pr-4 text-sm text-[var(--color-text-muted)]">{formattedDate}</td>
      <td className="py-3 pr-4 text-sm">{formatPrice(sale.amount_cents)}</td>
      <td className="py-3 pr-4 text-sm text-[var(--color-success)]">
        {formatPrice(sale.payout_cents)}
      </td>
      <td className="py-3">
        <span
          className={`text-xs px-2 py-0.5 rounded-full ${
            sale.payout_status === 'paid'
              ? 'bg-[var(--color-success)]/15 text-[var(--color-success)]'
              : sale.payout_status === 'pending'
                ? 'bg-[var(--color-accent-secondary)]/15 text-[var(--color-accent-secondary)]'
                : 'bg-[var(--color-error)]/15 text-[var(--color-error)]'
          }`}
        >
          {sale.payout_status}
        </span>
      </td>
    </tr>
  );
}

function BecomeSellerCTA() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleBecomeSeller = async () => {
    setIsLoading(true);
    setError(null);
    try {
      const { onboarding_url } = await becomeSeller();
      window.location.href = onboarding_url;
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start onboarding');
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <motion.div
      className="card text-center py-12"
      initial={{ opacity: 0, y: 20 }}
      animate={{ opacity: 1, y: 0 }}
    >
      <Package className="w-12 h-12 mx-auto text-[var(--color-accent)] mb-4" />
      <h2 className="text-xl font-semibold mb-2">Become a Seller</h2>
      <p className="text-[var(--color-text-muted)] mb-6 max-w-md mx-auto">
        Share your skills with the FGP community and earn money. Set up Stripe Connect to receive
        payouts directly to your bank account.
      </p>
      {error && (
        <div className="flex items-center justify-center gap-2 text-[var(--color-error)] mb-4">
          <AlertCircle className="w-4 h-4" />
          <span className="text-sm">{error}</span>
        </div>
      )}
      <button onClick={handleBecomeSeller} disabled={isLoading} className="btn btn-primary">
        {isLoading ? (
          <>
            <Loader2 className="w-4 h-4 animate-spin mr-2" />
            Setting up...
          </>
        ) : (
          'Set Up Stripe Connect'
        )}
      </button>
      <p className="text-xs text-[var(--color-text-muted)] mt-4">
        You'll be redirected to Stripe to complete onboarding
      </p>
    </motion.div>
  );
}

function SellerDashboard() {
  const { user, isAuthenticated, isLoading: authLoading } = useAuth();
  const navigate = useNavigate();
  const [stats, setStats] = useState<SellerStats | null>(null);
  const [skills, setSkills] = useState<SellerSkill[]>([]);
  const [sales, setSales] = useState<Sale[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      navigate({ to: '/marketplace' });
      return;
    }

    if (isAuthenticated && user?.role === 'seller') {
      loadSellerData();
    } else if (isAuthenticated) {
      setIsLoading(false);
    }
  }, [isAuthenticated, authLoading, user, navigate]);

  const loadSellerData = async () => {
    try {
      const [statsData, skillsData, salesData] = await Promise.all([
        fetchSellerStats(),
        fetchSellerSkills(),
        fetchSellerSales({ limit: 10 }),
      ]);
      setStats(statsData);
      setSkills(skillsData);
      setSales(salesData.sales);
    } catch (err) {
      setError('Failed to load seller data');
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

  // User is not a seller yet - show CTA
  if (user?.role !== 'seller') {
    return (
      <div className="section">
        <div className="container max-w-2xl">
          <motion.div
            className="mb-8"
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
          >
            <h1 className="text-section">Seller Dashboard</h1>
            <p className="text-[var(--color-text-secondary)] mt-2">
              Sell your skills on the FGP Marketplace
            </p>
          </motion.div>
          <BecomeSellerCTA />
        </div>
      </div>
    );
  }

  return (
    <div className="section">
      <div className="container max-w-6xl">
        <motion.div
          className="flex items-center justify-between gap-4 mb-8"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <div>
            <h1 className="text-section">Seller Dashboard</h1>
            <p className="text-[var(--color-text-secondary)] mt-2">
              Manage your skills and track earnings
            </p>
          </div>
          <Link to="/seller/submit" className="btn btn-primary">
            <Plus className="w-4 h-4 mr-2" />
            Submit Skill
          </Link>
        </motion.div>

        {isLoading ? (
          <div className="flex items-center justify-center py-12">
            <Loader2 className="w-8 h-8 animate-spin text-[var(--color-accent)]" />
          </div>
        ) : error ? (
          <div className="card text-center py-12">
            <AlertCircle className="w-8 h-8 mx-auto text-[var(--color-error)] mb-4" />
            <p className="text-[var(--color-error)]">{error}</p>
            <button onClick={loadSellerData} className="btn btn-secondary mt-4">
              Try Again
            </button>
          </div>
        ) : (
          <>
            {/* Stats Grid */}
            <motion.div
              className="grid grid-cols-2 lg:grid-cols-4 gap-4 mb-8"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.1 }}
            >
              <StatCard
                label="Total Sales"
                value={formatPrice(stats?.total_sales_cents || 0)}
                icon={DollarSign}
                color="var(--color-success)"
              />
              <StatCard
                label="Pending Payout"
                value={formatPrice(stats?.pending_payout_cents || 0)}
                icon={Clock}
                color="var(--color-accent-secondary)"
              />
              <StatCard
                label="Total Downloads"
                value={stats?.total_downloads.toLocaleString() || '0'}
                icon={Download}
                color="var(--color-accent)"
              />
              <StatCard
                label="Active Skills"
                value={`${stats?.active_skills || 0} / ${stats?.total_skills || 0}`}
                icon={Package}
              />
            </motion.div>

            {/* Skills List */}
            <motion.div
              className="mb-8"
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.2 }}
            >
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-lg font-semibold">Your Skills</h2>
              </div>
              {skills.length === 0 ? (
                <div className="card text-center py-8">
                  <Package className="w-8 h-8 mx-auto text-[var(--color-text-muted)] mb-3" />
                  <p className="text-[var(--color-text-muted)]">No skills submitted yet</p>
                  <Link to="/seller/submit" className="btn btn-secondary mt-4">
                    Submit Your First Skill
                  </Link>
                </div>
              ) : (
                <div className="space-y-3">
                  {skills.map((skill) => (
                    <SkillCard key={skill.id} skill={skill} />
                  ))}
                </div>
              )}
            </motion.div>

            {/* Recent Sales */}
            <motion.div
              initial={{ opacity: 0, y: 20 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: 0.3 }}
            >
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-lg font-semibold">Recent Sales</h2>
                <TrendingUp className="w-5 h-5 text-[var(--color-text-muted)]" />
              </div>
              {sales.length === 0 ? (
                <div className="card text-center py-8">
                  <DollarSign className="w-8 h-8 mx-auto text-[var(--color-text-muted)] mb-3" />
                  <p className="text-[var(--color-text-muted)]">No sales yet</p>
                </div>
              ) : (
                <div className="card overflow-x-auto">
                  <table className="w-full">
                    <thead>
                      <tr className="text-left text-sm text-[var(--color-text-muted)] border-b border-[var(--color-border)]">
                        <th className="pb-3 pr-4 font-medium">Skill</th>
                        <th className="pb-3 pr-4 font-medium">Date</th>
                        <th className="pb-3 pr-4 font-medium">Sale</th>
                        <th className="pb-3 pr-4 font-medium">Your Payout</th>
                        <th className="pb-3 font-medium">Status</th>
                      </tr>
                    </thead>
                    <tbody>
                      {sales.map((sale) => (
                        <SaleRow key={sale.id} sale={sale} />
                      ))}
                    </tbody>
                  </table>
                </div>
              )}
            </motion.div>
          </>
        )}
      </div>
    </div>
  );
}

export const Route = createFileRoute('/seller/')({
  component: SellerDashboard,
});

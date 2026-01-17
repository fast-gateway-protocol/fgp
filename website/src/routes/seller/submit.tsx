import { createFileRoute, useNavigate } from '@tanstack/react-router';
import { motion } from 'framer-motion';
import {
  Package,
  Loader2,
  AlertCircle,
  Check,
  Info,
  DollarSign,
  Code,
  FileJson,
} from 'lucide-react';
import { useState, useEffect } from 'react';
import { useAuth } from '@/context/AuthContext';
import { submitSkill, type SkillSubmission } from '@/lib/api';
import { categories } from '@/data/registry';

const tiers = [
  {
    id: 'free',
    name: 'Free',
    description: 'Open source, community contribution',
    color: 'var(--color-success)',
    platformFee: '0%',
  },
  {
    id: 'community',
    name: 'Community',
    description: '$5-29, 30% platform fee',
    color: 'var(--color-accent-secondary)',
    platformFee: '30%',
    minPrice: 500,
    maxPrice: 2900,
  },
  {
    id: 'verified',
    name: 'Verified',
    description: '$30-99, 25% platform fee (requires review)',
    color: 'var(--color-accent)',
    platformFee: '25%',
    minPrice: 3000,
    maxPrice: 9900,
  },
  {
    id: 'pro',
    name: 'Pro',
    description: '$100-499, 20% platform fee (premium support)',
    color: '#a855f7',
    platformFee: '20%',
    minPrice: 10000,
    maxPrice: 49900,
  },
] as const;

const platforms = [
  { id: 'darwin', name: 'macOS' },
  { id: 'linux', name: 'Linux' },
  { id: 'win32', name: 'Windows' },
];

function slugify(text: string): string {
  return text
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');
}

function validateManifest(json: string): { valid: boolean; error?: string } {
  if (!json.trim()) {
    return { valid: false, error: 'Manifest is required' };
  }

  try {
    const manifest = JSON.parse(json);
    if (!manifest.name) {
      return { valid: false, error: 'Manifest must have a "name" field' };
    }
    if (!manifest.version) {
      return { valid: false, error: 'Manifest must have a "version" field' };
    }
    return { valid: true };
  } catch {
    return { valid: false, error: 'Invalid JSON format' };
  }
}

function SubmitSkillPage() {
  const { user, isAuthenticated, isLoading: authLoading } = useAuth();
  const navigate = useNavigate();

  const [formData, setFormData] = useState({
    name: '',
    slug: '',
    description: '',
    version: '1.0.0',
    tier: 'free' as 'free' | 'community' | 'verified' | 'pro',
    price_cents: 0,
    categories: [] as string[],
    platforms: ['darwin', 'linux'] as string[],
    repository_url: '',
    manifest_json: '',
  });

  const [errors, setErrors] = useState<Record<string, string>>({});
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [submitError, setSubmitError] = useState<string | null>(null);

  useEffect(() => {
    if (!authLoading && !isAuthenticated) {
      navigate({ to: '/marketplace' });
    }
    if (!authLoading && isAuthenticated && user?.role !== 'seller') {
      navigate({ to: '/seller' });
    }
  }, [isAuthenticated, authLoading, user, navigate]);

  const handleNameChange = (name: string) => {
    setFormData((prev) => ({
      ...prev,
      name,
      slug: slugify(name),
    }));
    setErrors((prev) => ({ ...prev, name: '', slug: '' }));
  };

  const handleTierChange = (tier: typeof formData.tier) => {
    const tierConfig = tiers.find((t) => t.id === tier);
    let newPrice = 0;
    if (tier !== 'free' && tierConfig && 'minPrice' in tierConfig) {
      newPrice = tierConfig.minPrice;
    }
    setFormData((prev) => ({
      ...prev,
      tier,
      price_cents: newPrice,
    }));
    setErrors((prev) => ({ ...prev, tier: '', price_cents: '' }));
  };

  const handleCategoryToggle = (categoryId: string) => {
    setFormData((prev) => ({
      ...prev,
      categories: prev.categories.includes(categoryId)
        ? prev.categories.filter((c) => c !== categoryId)
        : [...prev.categories, categoryId],
    }));
    setErrors((prev) => ({ ...prev, categories: '' }));
  };

  const handlePlatformToggle = (platformId: string) => {
    setFormData((prev) => ({
      ...prev,
      platforms: prev.platforms.includes(platformId)
        ? prev.platforms.filter((p) => p !== platformId)
        : [...prev.platforms, platformId],
    }));
    setErrors((prev) => ({ ...prev, platforms: '' }));
  };

  const validate = (): boolean => {
    const newErrors: Record<string, string> = {};

    if (!formData.name.trim()) {
      newErrors.name = 'Name is required';
    }

    if (!formData.slug.trim()) {
      newErrors.slug = 'Slug is required';
    } else if (!/^[a-z0-9-]+$/.test(formData.slug)) {
      newErrors.slug = 'Slug can only contain lowercase letters, numbers, and hyphens';
    }

    if (!formData.description.trim()) {
      newErrors.description = 'Description is required';
    } else if (formData.description.length < 20) {
      newErrors.description = 'Description must be at least 20 characters';
    }

    if (!formData.version.trim()) {
      newErrors.version = 'Version is required';
    } else if (!/^\d+\.\d+\.\d+$/.test(formData.version)) {
      newErrors.version = 'Version must be in semver format (e.g., 1.0.0)';
    }

    if (formData.tier !== 'free') {
      const tierConfig = tiers.find((t) => t.id === formData.tier);
      if (tierConfig && 'minPrice' in tierConfig) {
        if (formData.price_cents < tierConfig.minPrice) {
          newErrors.price_cents = `Price must be at least $${(tierConfig.minPrice / 100).toFixed(2)}`;
        }
        if (formData.price_cents > tierConfig.maxPrice) {
          newErrors.price_cents = `Price cannot exceed $${(tierConfig.maxPrice / 100).toFixed(2)}`;
        }
      }
    }

    if (formData.categories.length === 0) {
      newErrors.categories = 'Select at least one category';
    }

    if (formData.platforms.length === 0) {
      newErrors.platforms = 'Select at least one platform';
    }

    const manifestValidation = validateManifest(formData.manifest_json);
    if (!manifestValidation.valid) {
      newErrors.manifest_json = manifestValidation.error || 'Invalid manifest';
    }

    setErrors(newErrors);
    return Object.keys(newErrors).length === 0;
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();

    if (!validate()) return;

    setIsSubmitting(true);
    setSubmitError(null);

    try {
      const submission: SkillSubmission = {
        name: formData.name,
        slug: formData.slug,
        description: formData.description,
        version: formData.version,
        tier: formData.tier,
        price_cents: formData.tier === 'free' ? 0 : formData.price_cents,
        categories: formData.categories,
        platforms: formData.platforms,
        repository_url: formData.repository_url || undefined,
        manifest_json: formData.manifest_json,
      };

      await submitSkill(submission);
      navigate({ to: '/seller' });
    } catch (err) {
      setSubmitError(err instanceof Error ? err.message : 'Failed to submit skill');
    } finally {
      setIsSubmitting(false);
    }
  };

  if (authLoading) {
    return (
      <div className="section min-h-[60vh] flex items-center justify-center">
        <Loader2 className="w-8 h-8 animate-spin text-[var(--color-accent)]" />
      </div>
    );
  }

  const selectedTier = tiers.find((t) => t.id === formData.tier);

  return (
    <div className="section">
      <div className="container max-w-3xl">
        <motion.div
          className="mb-8"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
        >
          <h1 className="text-section">Submit a Skill</h1>
          <p className="text-[var(--color-text-secondary)] mt-2">
            Share your skill with the FGP community
          </p>
        </motion.div>

        <motion.form
          onSubmit={handleSubmit}
          className="space-y-6"
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          transition={{ delay: 0.1 }}
        >
          {/* Basic Info */}
          <div className="card">
            <h2 className="font-semibold mb-4 flex items-center gap-2">
              <Package className="w-5 h-5" />
              Basic Information
            </h2>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium mb-1">Name *</label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => handleNameChange(e.target.value)}
                  placeholder="e.g., Twitter Research"
                  className={`input ${errors.name ? 'border-[var(--color-error)]' : ''}`}
                />
                {errors.name && (
                  <p className="text-xs text-[var(--color-error)] mt-1">{errors.name}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Slug *</label>
                <input
                  type="text"
                  value={formData.slug}
                  onChange={(e) => setFormData((prev) => ({ ...prev, slug: e.target.value }))}
                  placeholder="e.g., twitter-research"
                  className={`input ${errors.slug ? 'border-[var(--color-error)]' : ''}`}
                />
                <p className="text-xs text-[var(--color-text-muted)] mt-1">
                  URL: fgp.dev/marketplace/{formData.slug || 'your-skill'}
                </p>
                {errors.slug && (
                  <p className="text-xs text-[var(--color-error)] mt-1">{errors.slug}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Description *</label>
                <textarea
                  value={formData.description}
                  onChange={(e) =>
                    setFormData((prev) => ({ ...prev, description: e.target.value }))
                  }
                  placeholder="Describe what your skill does and why it's useful..."
                  rows={3}
                  className={`input resize-none ${errors.description ? 'border-[var(--color-error)]' : ''}`}
                />
                {errors.description && (
                  <p className="text-xs text-[var(--color-error)] mt-1">{errors.description}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium mb-1">Version *</label>
                <input
                  type="text"
                  value={formData.version}
                  onChange={(e) => setFormData((prev) => ({ ...prev, version: e.target.value }))}
                  placeholder="1.0.0"
                  className={`input max-w-[150px] ${errors.version ? 'border-[var(--color-error)]' : ''}`}
                />
                {errors.version && (
                  <p className="text-xs text-[var(--color-error)] mt-1">{errors.version}</p>
                )}
              </div>
            </div>
          </div>

          {/* Pricing */}
          <div className="card">
            <h2 className="font-semibold mb-4 flex items-center gap-2">
              <DollarSign className="w-5 h-5" />
              Pricing
            </h2>

            <div className="grid grid-cols-2 md:grid-cols-4 gap-3 mb-4">
              {tiers.map((tier) => (
                <button
                  key={tier.id}
                  type="button"
                  onClick={() => handleTierChange(tier.id as typeof formData.tier)}
                  className={`p-3 rounded-lg border-2 text-left transition-all ${
                    formData.tier === tier.id
                      ? 'border-current'
                      : 'border-[var(--color-border)] hover:border-[var(--color-border-hover)]'
                  }`}
                  style={{ color: formData.tier === tier.id ? tier.color : undefined }}
                >
                  <div className="font-medium">{tier.name}</div>
                  <div className="text-xs text-[var(--color-text-muted)] mt-1">
                    {tier.platformFee} fee
                  </div>
                </button>
              ))}
            </div>

            {formData.tier !== 'free' && selectedTier && 'minPrice' in selectedTier && (
              <div className="mt-4">
                <label className="block text-sm font-medium mb-1">
                  Price (USD) *
                  <span className="text-[var(--color-text-muted)] font-normal ml-2">
                    ${(selectedTier.minPrice / 100).toFixed(0)} - $
                    {(selectedTier.maxPrice / 100).toFixed(0)}
                  </span>
                </label>
                <div className="relative max-w-[200px]">
                  <span className="absolute left-3 top-1/2 -translate-y-1/2 text-[var(--color-text-muted)]">
                    $
                  </span>
                  <input
                    type="number"
                    value={(formData.price_cents / 100).toFixed(2)}
                    onChange={(e) =>
                      setFormData((prev) => ({
                        ...prev,
                        price_cents: Math.round(parseFloat(e.target.value || '0') * 100),
                      }))
                    }
                    step="0.01"
                    min={selectedTier.minPrice / 100}
                    max={selectedTier.maxPrice / 100}
                    className={`input pl-7 ${errors.price_cents ? 'border-[var(--color-error)]' : ''}`}
                  />
                </div>
                {errors.price_cents && (
                  <p className="text-xs text-[var(--color-error)] mt-1">{errors.price_cents}</p>
                )}
                <p className="text-xs text-[var(--color-text-muted)] mt-2">
                  You'll receive{' '}
                  <span className="text-[var(--color-success)]">
                    ${((formData.price_cents / 100) * (1 - parseInt(selectedTier.platformFee) / 100)).toFixed(2)}
                  </span>{' '}
                  per sale ({100 - parseInt(selectedTier.platformFee)}% after platform fee + Stripe
                  processing)
                </p>
              </div>
            )}
          </div>

          {/* Categories */}
          <div className="card">
            <h2 className="font-semibold mb-4">Categories *</h2>
            <div className="flex flex-wrap gap-2">
              {categories.map((category) => (
                <button
                  key={category.id}
                  type="button"
                  onClick={() => handleCategoryToggle(category.id)}
                  className={`px-3 py-1.5 rounded-full text-sm transition-all flex items-center gap-1.5 ${
                    formData.categories.includes(category.id)
                      ? 'bg-[var(--color-accent)] text-[var(--color-void)]'
                      : 'bg-[var(--color-surface-elevated)] text-[var(--color-text-secondary)] hover:bg-[var(--color-surface-hover)]'
                  }`}
                >
                  {formData.categories.includes(category.id) && <Check className="w-3 h-3" />}
                  {category.name}
                </button>
              ))}
            </div>
            {errors.categories && (
              <p className="text-xs text-[var(--color-error)] mt-2">{errors.categories}</p>
            )}
          </div>

          {/* Platforms */}
          <div className="card">
            <h2 className="font-semibold mb-4">Supported Platforms *</h2>
            <div className="flex flex-wrap gap-3">
              {platforms.map((platform) => (
                <label key={platform.id} className="flex items-center gap-2 cursor-pointer">
                  <input
                    type="checkbox"
                    checked={formData.platforms.includes(platform.id)}
                    onChange={() => handlePlatformToggle(platform.id)}
                    className="w-4 h-4 rounded border-[var(--color-border)] bg-[var(--color-surface)] text-[var(--color-accent)] focus:ring-[var(--color-accent)]"
                  />
                  <span className="text-sm">{platform.name}</span>
                </label>
              ))}
            </div>
            {errors.platforms && (
              <p className="text-xs text-[var(--color-error)] mt-2">{errors.platforms}</p>
            )}
          </div>

          {/* Repository URL */}
          <div className="card">
            <h2 className="font-semibold mb-4 flex items-center gap-2">
              <Code className="w-5 h-5" />
              Source Code (Optional)
            </h2>
            <input
              type="url"
              value={formData.repository_url}
              onChange={(e) => setFormData((prev) => ({ ...prev, repository_url: e.target.value }))}
              placeholder="https://github.com/username/skill-repo"
              className="input"
            />
            <p className="text-xs text-[var(--color-text-muted)] mt-1">
              Public repository URL (increases trust and downloads)
            </p>
          </div>

          {/* Manifest JSON */}
          <div className="card">
            <h2 className="font-semibold mb-4 flex items-center gap-2">
              <FileJson className="w-5 h-5" />
              Skill Manifest *
            </h2>
            <div className="bg-[var(--color-surface-elevated)] rounded-lg p-3 mb-3 flex items-start gap-2">
              <Info className="w-4 h-4 text-[var(--color-accent)] flex-shrink-0 mt-0.5" />
              <p className="text-xs text-[var(--color-text-muted)]">
                Paste your skill.json manifest. Must include "name", "version", and other required
                fields. See the{' '}
                <a
                  href="/docs/skills"
                  target="_blank"
                  className="text-[var(--color-accent)] hover:underline"
                >
                  skill documentation
                </a>{' '}
                for the full schema.
              </p>
            </div>
            <textarea
              value={formData.manifest_json}
              onChange={(e) =>
                setFormData((prev) => ({ ...prev, manifest_json: e.target.value }))
              }
              placeholder={`{
  "name": "your-skill",
  "version": "1.0.0",
  "description": "Your skill description",
  "methods": [...]
}`}
              rows={12}
              className={`input font-mono text-sm resize-none ${errors.manifest_json ? 'border-[var(--color-error)]' : ''}`}
            />
            {errors.manifest_json && (
              <p className="text-xs text-[var(--color-error)] mt-1">{errors.manifest_json}</p>
            )}
          </div>

          {/* Submit */}
          {submitError && (
            <div className="flex items-center gap-2 text-[var(--color-error)] bg-[var(--color-error)]/10 rounded-lg p-3">
              <AlertCircle className="w-5 h-5 flex-shrink-0" />
              <span className="text-sm">{submitError}</span>
            </div>
          )}

          <div className="flex items-center justify-between gap-4">
            <button
              type="button"
              onClick={() => navigate({ to: '/seller' })}
              className="btn btn-secondary"
            >
              Cancel
            </button>
            <button type="submit" disabled={isSubmitting} className="btn btn-primary">
              {isSubmitting ? (
                <>
                  <Loader2 className="w-4 h-4 animate-spin mr-2" />
                  Submitting...
                </>
              ) : (
                'Submit for Review'
              )}
            </button>
          </div>

          <p className="text-xs text-[var(--color-text-muted)] text-center">
            By submitting, you agree to the{' '}
            <a href="/docs/seller-terms" className="text-[var(--color-accent)] hover:underline">
              Seller Terms of Service
            </a>
          </p>
        </motion.form>
      </div>
    </div>
  );
}

export const Route = createFileRoute('/seller/submit')({
  component: SubmitSkillPage,
});

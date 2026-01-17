// FGP Marketplace API client

const API_BASE = import.meta.env.VITE_API_URL || 'https://api.fgp.dev/v1';

// Types
export interface MarketplaceSkill {
  id: string;
  slug: string;
  name: string;
  description: string;
  version: string;
  tier: 'free' | 'community' | 'verified' | 'pro';
  price_cents: number;
  currency: string;
  seller_id: string;
  seller_name?: string;
  seller_avatar?: string;
  status: 'active' | 'pending' | 'archived';
  download_count: number;
  rating_average: number | null;
  rating_count: number;
  categories: string[];
  platforms: string[];
  repository_url: string | null;
  preview_images: string[];
  created_at: string;
  updated_at: string;
}

export interface User {
  id: string;
  email: string;
  name: string | null;
  github_username: string | null;
  avatar_url: string | null;
  role: 'user' | 'seller' | 'admin';
  stripe_connect_id: string | null;
  created_at: string;
}

export interface Purchase {
  id: string;
  license_key: string;
  license_status: 'active' | 'revoked' | 'refunded';
  amount_cents: number;
  purchased_at: string;
  skill: {
    id: string;
    name: string;
    slug: string;
    version: string;
    description: string;
  };
}

export interface CheckoutResponse {
  checkout_url: string;
  session_id: string;
}

// Seller types
export interface SellerSkill {
  id: string;
  slug: string;
  name: string;
  description: string;
  version: string;
  tier: 'free' | 'community' | 'verified' | 'pro';
  price_cents: number;
  currency: string;
  status: 'pending' | 'active' | 'rejected' | 'archived';
  download_count: number;
  rating_average: number | null;
  rating_count: number;
  categories: string[];
  platforms: string[];
  repository_url: string | null;
  total_sales_cents: number;
  total_payout_cents: number;
  created_at: string;
  updated_at: string;
}

export interface SellerStats {
  total_skills: number;
  active_skills: number;
  total_sales_cents: number;
  total_payout_cents: number;
  pending_payout_cents: number;
  total_downloads: number;
}

export interface Sale {
  id: string;
  skill_name: string;
  skill_slug: string;
  amount_cents: number;
  payout_cents: number;
  payout_status: 'pending' | 'paid' | 'failed';
  purchased_at: string;
}

export interface SkillSubmission {
  name: string;
  slug: string;
  description: string;
  version: string;
  tier: 'free' | 'community' | 'verified' | 'pro';
  price_cents: number;
  categories: string[];
  platforms: string[];
  repository_url?: string;
  manifest_json: string;
}

export interface StripeConnectResponse {
  onboarding_url: string;
  account_id: string;
}

// API Functions

export async function fetchSkills(params?: {
  category?: string;
  tier?: string;
  search?: string;
  limit?: number;
  offset?: number;
}): Promise<{ skills: MarketplaceSkill[]; total: number }> {
  const searchParams = new URLSearchParams();
  if (params?.category) searchParams.set('category', params.category);
  if (params?.tier) searchParams.set('tier', params.tier);
  if (params?.search) searchParams.set('search', params.search);
  if (params?.limit) searchParams.set('limit', params.limit.toString());
  if (params?.offset) searchParams.set('offset', params.offset.toString());

  const url = `${API_BASE}/skills?${searchParams}`;
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error('Failed to fetch skills');
  }

  return response.json();
}

export async function fetchSkill(slug: string): Promise<MarketplaceSkill | null> {
  const response = await fetch(`${API_BASE}/skills/${slug}`);

  if (response.status === 404) {
    return null;
  }

  if (!response.ok) {
    throw new Error('Failed to fetch skill');
  }

  return response.json();
}

export async function fetchCurrentUser(): Promise<User | null> {
  const response = await fetch(`${API_BASE}/auth/me`, {
    credentials: 'include',
  });

  if (!response.ok) {
    return null;
  }

  const data = await response.json();
  return data.user;
}

export async function logout(): Promise<void> {
  await fetch(`${API_BASE}/auth/logout`, {
    method: 'POST',
    credentials: 'include',
  });
}

export async function createCheckout(skillSlug: string): Promise<CheckoutResponse> {
  const response = await fetch(`${API_BASE}/purchases/${skillSlug}/checkout`, {
    method: 'POST',
    credentials: 'include',
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to create checkout');
  }

  return response.json();
}

export async function fetchPurchases(): Promise<Purchase[]> {
  const response = await fetch(`${API_BASE}/purchases`, {
    credentials: 'include',
  });

  if (!response.ok) {
    throw new Error('Failed to fetch purchases');
  }

  const data = await response.json();
  return data.purchases;
}

export async function fetchPurchase(id: string): Promise<Purchase | null> {
  const response = await fetch(`${API_BASE}/purchases/${id}`, {
    credentials: 'include',
  });

  if (response.status === 404) {
    return null;
  }

  if (!response.ok) {
    throw new Error('Failed to fetch purchase');
  }

  return response.json();
}

export function getGitHubLoginUrl(): string {
  return `${API_BASE}/auth/github`;
}

// Utility functions

export function formatPrice(priceCents: number, currency: string = 'USD'): string {
  const dollars = priceCents / 100;
  return new Intl.NumberFormat('en-US', {
    style: 'currency',
    currency,
  }).format(dollars);
}

export function getTierColor(tier: string): string {
  switch (tier) {
    case 'free':
      return 'var(--color-success)';
    case 'community':
      return 'var(--color-accent-secondary)';
    case 'verified':
      return 'var(--color-accent)';
    case 'pro':
      return '#a855f7'; // Purple
    default:
      return 'var(--color-text-muted)';
  }
}

export function getTierLabel(tier: string): string {
  switch (tier) {
    case 'free':
      return 'Free';
    case 'community':
      return 'Community';
    case 'verified':
      return 'Verified';
    case 'pro':
      return 'Pro';
    default:
      return tier;
  }
}

// Seller API Functions

export async function becomeSeller(): Promise<StripeConnectResponse> {
  const response = await fetch(`${API_BASE}/auth/become-seller`, {
    method: 'POST',
    credentials: 'include',
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to start seller onboarding');
  }

  return response.json();
}

export async function fetchSellerStats(): Promise<SellerStats> {
  const response = await fetch(`${API_BASE}/seller/stats`, {
    credentials: 'include',
  });

  if (!response.ok) {
    throw new Error('Failed to fetch seller stats');
  }

  return response.json();
}

export async function fetchSellerSkills(): Promise<SellerSkill[]> {
  const response = await fetch(`${API_BASE}/seller/skills`, {
    credentials: 'include',
  });

  if (!response.ok) {
    throw new Error('Failed to fetch seller skills');
  }

  const data = await response.json();
  return data.skills;
}

export async function fetchSellerSales(params?: {
  limit?: number;
  offset?: number;
}): Promise<{ sales: Sale[]; total: number }> {
  const searchParams = new URLSearchParams();
  if (params?.limit) searchParams.set('limit', params.limit.toString());
  if (params?.offset) searchParams.set('offset', params.offset.toString());

  const response = await fetch(`${API_BASE}/seller/sales?${searchParams}`, {
    credentials: 'include',
  });

  if (!response.ok) {
    throw new Error('Failed to fetch sales');
  }

  return response.json();
}

export async function submitSkill(submission: SkillSubmission): Promise<SellerSkill> {
  const response = await fetch(`${API_BASE}/seller/skills`, {
    method: 'POST',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(submission),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to submit skill');
  }

  return response.json();
}

export async function updateSkill(
  skillId: string,
  updates: Partial<SkillSubmission>
): Promise<SellerSkill> {
  const response = await fetch(`${API_BASE}/seller/skills/${skillId}`, {
    method: 'PATCH',
    credentials: 'include',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(updates),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to update skill');
  }

  return response.json();
}

export async function archiveSkill(skillId: string): Promise<void> {
  const response = await fetch(`${API_BASE}/seller/skills/${skillId}/archive`, {
    method: 'POST',
    credentials: 'include',
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || 'Failed to archive skill');
  }
}

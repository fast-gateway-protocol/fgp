import { afterEach, describe, expect, it, vi } from 'vitest';

import {
  archiveSkill,
  becomeSeller,
  createCheckout,
  fetchCurrentUser,
  fetchPurchase,
  fetchPurchases,
  fetchSellerSales,
  fetchSellerSkills,
  fetchSellerStats,
  fetchSkill,
  fetchSkills,
  formatPrice,
  getGitHubLoginUrl,
  getTierColor,
  getTierLabel,
  logout,
  submitSkill,
  updateSkill,
} from './api';

afterEach(() => {
  vi.restoreAllMocks();
});

describe('formatPrice', () => {
  it('formats cents into USD by default', () => {
    expect(formatPrice(250)).toBe('$2.50');
  });

  it('formats cents for provided currency', () => {
    expect(formatPrice(1234, 'EUR')).toBe('€12.34');
  });
});

describe('getTierColor', () => {
  it('maps known tiers to color tokens', () => {
    expect(getTierColor('free')).toBe('var(--color-success)');
    expect(getTierColor('community')).toBe('var(--color-accent-secondary)');
    expect(getTierColor('verified')).toBe('var(--color-accent)');
    expect(getTierColor('pro')).toBe('#a855f7');
  });

  it('falls back for unknown tiers', () => {
    expect(getTierColor('unknown')).toBe('var(--color-text-muted)');
  });
});

describe('getTierLabel', () => {
  it('maps known tiers to labels', () => {
    expect(getTierLabel('free')).toBe('Free');
    expect(getTierLabel('community')).toBe('Community');
    expect(getTierLabel('verified')).toBe('Verified');
    expect(getTierLabel('pro')).toBe('Pro');
  });

  it('returns the tier for unknown labels', () => {
    expect(getTierLabel('enterprise')).toBe('enterprise');
  });
});

describe('fetchSkills', () => {
  it('builds query params and returns payload', async () => {
    const payload = { skills: [], total: 0 };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);

    const result = await fetchSkills({ category: 'tools', tier: 'free', limit: 5 });
    expect(result).toEqual(payload);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    const url = String(fetchMock.mock.calls[0][0]);
    expect(url).toContain('https://api.fgp.dev/v1/skills?');
    expect(url).toContain('category=tools');
    expect(url).toContain('tier=free');
    expect(url).toContain('limit=5');
  });

  it('throws on non-ok responses', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: false });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchSkills()).rejects.toThrow('Failed to fetch skills');
  });
});

describe('fetchSkill', () => {
  it('returns null when the skill is not found', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ status: 404, ok: false });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchSkill('missing')).resolves.toBeNull();
  });

  it('returns the skill payload when available', async () => {
    const payload = { id: '1', slug: 'demo' };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchSkill('demo')).resolves.toEqual(payload);
  });
});

describe('fetchCurrentUser', () => {
  it('returns null when request fails', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: false });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchCurrentUser()).resolves.toBeNull();
  });

  it('returns user when request succeeds', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve({ user: { id: 'user-1' } }),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchCurrentUser()).resolves.toEqual({ id: 'user-1' });
  });
});

describe('createCheckout', () => {
  it('throws error message from response payload', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: () => Promise.resolve({ error: 'Nope' }),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(createCheckout('demo')).rejects.toThrow('Nope');
  });

  it('returns checkout response when ok', async () => {
    const payload = { checkout_url: 'https://example.com', session_id: 'sess' };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(createCheckout('demo')).resolves.toEqual(payload);
  });
});

describe('logout', () => {
  it('posts to logout endpoint', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ ok: true });
    vi.stubGlobal('fetch', fetchMock);
    await logout();
    expect(fetchMock).toHaveBeenCalledWith('https://api.fgp.dev/v1/auth/logout', {
      method: 'POST',
      credentials: 'include',
    });
  });
});

describe('fetchPurchases', () => {
  it('returns purchases list', async () => {
    const payload = { purchases: [{ id: '1' }] };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchPurchases()).resolves.toEqual(payload.purchases);
  });
});

describe('fetchPurchase', () => {
  it('returns null on 404', async () => {
    const fetchMock = vi.fn().mockResolvedValue({ status: 404, ok: false });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchPurchase('missing')).resolves.toBeNull();
  });

  it('returns purchase payload when ok', async () => {
    const payload = { id: 'p1' };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      status: 200,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchPurchase('p1')).resolves.toEqual(payload);
  });
});

describe('getGitHubLoginUrl', () => {
  it('returns auth url', () => {
    expect(getGitHubLoginUrl()).toBe('https://api.fgp.dev/v1/auth/github');
  });
});

describe('seller endpoints', () => {
  it('fetches seller stats', async () => {
    const payload = { total_skills: 1 };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchSellerStats()).resolves.toEqual(payload);
  });

  it('fetches seller skills list', async () => {
    const payload = { skills: [{ id: '1' }] };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchSellerSkills()).resolves.toEqual(payload.skills);
  });

  it('fetches seller sales with params', async () => {
    const payload = { sales: [], total: 0 };
    const fetchMock = vi.fn().mockResolvedValue({
      ok: true,
      json: () => Promise.resolve(payload),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(fetchSellerSales({ limit: 5, offset: 10 })).resolves.toEqual(payload);
    const url = String(fetchMock.mock.calls[0][0]);
    expect(url).toContain('limit=5');
    expect(url).toContain('offset=10');
  });

  it('becomeSeller surfaces error response', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: () => Promise.resolve({ error: 'Denied' }),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(becomeSeller()).rejects.toThrow('Denied');
  });

  it('submitSkill surfaces error response', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: () => Promise.resolve({ error: 'Invalid' }),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(
      submitSkill({
        name: 'Demo',
        slug: 'demo',
        description: 'desc',
        version: '1.0.0',
        tier: 'free',
        price_cents: 0,
        categories: [],
        platforms: [],
        manifest_json: '{}',
      })
    ).rejects.toThrow('Invalid');
  });

  it('updateSkill surfaces error response', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: () => Promise.resolve({ error: 'Invalid' }),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(updateSkill('skill-1', { name: 'Demo' })).rejects.toThrow('Invalid');
  });

  it('archiveSkill surfaces error response', async () => {
    const fetchMock = vi.fn().mockResolvedValue({
      ok: false,
      json: () => Promise.resolve({ error: 'Nope' }),
    });
    vi.stubGlobal('fetch', fetchMock);
    await expect(archiveSkill('skill-1')).rejects.toThrow('Nope');
  });
});

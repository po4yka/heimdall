import { describe, expect, it } from 'vitest';
import { mergeWithDefaults } from '../src/storage';
import { DEFAULT_CONFIG } from '../src/types';

describe('mergeWithDefaults', () => {
  it('returns defaults for null', () => {
    expect(mergeWithDefaults(null)).toEqual(DEFAULT_CONFIG);
  });

  it('preserves stored token', () => {
    const merged = mergeWithDefaults({ companionToken: 'abc'.repeat(22) + 'ab' });
    expect(merged.companionToken).toBe('abc'.repeat(22) + 'ab');
  });

  it('rejects negative intervals', () => {
    const merged = mergeWithDefaults({ syncIntervalMinutes: -5 });
    expect(merged.syncIntervalMinutes).toBe(DEFAULT_CONFIG.syncIntervalMinutes);
  });

  it('preserves per-vendor lastSeen state', () => {
    const merged = mergeWithDefaults({
      vendors: { 'claude.ai': { enabled: false, lastSyncAt: '2026-01-01', lastSeenUpdatedAt: { c1: '2026' } } },
    });
    expect(merged.vendors['claude.ai']?.enabled).toBe(false);
    expect(merged.vendors['claude.ai']?.lastSeenUpdatedAt['c1']).toBe('2026');
  });
});

import { describe, expect, it } from 'vitest';
import { getScopeBadgeColor } from './utils';

describe('getScopeBadgeColor', () => {
  it('renders Codex skills with their dedicated badge', () => {
    expect(getScopeBadgeColor('codex')).toContain('emerald');
  });

  it('matches scopes without regard to case', () => {
    expect(getScopeBadgeColor('CODEX')).toBe(getScopeBadgeColor('codex'));
  });
});

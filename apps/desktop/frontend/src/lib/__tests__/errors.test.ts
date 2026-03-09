import { describe, expect, it } from 'vitest';
import { toAppError, sanitizeMessage } from '../errors';

describe('toAppError', () => {
  it('handles null without throwing', () => {
    const r = toAppError(null);
    expect(r.kind).toBe('Unknown');
    expect(typeof r.message).toBe('string');
  });

  it('handles undefined without throwing', () => {
    const r = toAppError(undefined);
    expect(r.kind).toBe('Unknown');
  });

  it('wraps a plain string', () => {
    const r = toAppError('connection refused');
    expect(r.kind).toBe('Unknown');
    expect(r.message).toBe('connection refused');
  });

  it('preserves known kind from backend error', () => {
    const r = toAppError({ kind: 'Auth', message: 'invalid credentials' });
    expect(r.kind).toBe('Auth');
    expect(r.message).toBe('invalid credentials');
  });

  it('maps unknown kind to Unknown', () => {
    const r = toAppError({ kind: 'SomeNewKind', message: 'oops' });
    expect(r.kind).toBe('Unknown');
  });

  it('handles numeric error without throwing', () => {
    const r = toAppError(42);
    expect(r.kind).toBe('Unknown');
  });
});

describe('sanitizeMessage', () => {
  it('redacts password= pattern', () => {
    const out = sanitizeMessage('failed: password=MySecret123');
    expect(out).not.toContain('MySecret123');
    expect(out).toContain('[REDACTED]');
  });

  it('passes through non-sensitive messages unchanged', () => {
    const msg = 'connection timeout after 30s';
    expect(sanitizeMessage(msg)).toBe(msg);
  });

  it('redacts token: pattern', () => {
    const out = sanitizeMessage('auth token: abc123def');
    expect(out).not.toContain('abc123def');
  });
});

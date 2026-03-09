export type AppErrorKind =
  | 'Network'
  | 'Auth'
  | 'NotFound'
  | 'Validation'
  | 'Internal'
  | 'Database'
  | 'Crypto'
  | 'Sftp'
  | 'Ssh'
  | 'SetupRequired'
  | 'Unknown';

export interface AppError {
  kind: AppErrorKind;
  message: string;
}

const SENSITIVE_PATTERNS: RegExp[] = [
  /password[=:]\s*\S+/gi,
  /passwd[=:]\s*\S+/gi,
  /secret[=:]\s*\S+/gi,
  /token[=:]\s*\S+/gi,
];

export function sanitizeMessage(msg: string): string {
  return SENSITIVE_PATTERNS.reduce(
    (acc, re) => acc.replace(re, (m) => m.replace(/([=:])\s*\S+/, '$1[REDACTED]')),
    msg,
  );
}

const KIND_MAP: Partial<Record<string, AppErrorKind>> = {
  Network: 'Network',
  Unauthorized: 'Auth',
  Forbidden: 'Auth',
  Auth: 'Auth',
  NotFound: 'NotFound',
  Validation: 'Validation',
  Conflict: 'Validation',
  Internal: 'Internal',
  Database: 'Database',
  Crypto: 'Crypto',
  Ssh: 'Ssh',
  Sftp: 'Sftp',
  SetupRequired: 'SetupRequired',
};

export function toAppError(err: unknown): AppError {
  if (err !== null && typeof err === 'object') {
    const e = err as Record<string, unknown>;
    const kind: AppErrorKind =
      typeof e.kind === 'string' ? (KIND_MAP[e.kind] ?? 'Unknown') : 'Unknown';
    const message =
      typeof e.message === 'string' ? sanitizeMessage(e.message) : String(err);
    return { kind, message };
  }
  if (typeof err === 'string') {
    return { kind: 'Unknown', message: sanitizeMessage(err) };
  }
  return { kind: 'Unknown', message: 'An unknown error occurred' };
}

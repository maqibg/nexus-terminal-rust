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

const SFTP_KIND_LABELS: Record<string, string> = {
  'not-found': '不存在',
  'permission-denied': '权限不足',
  connection: '连接已断开',
  protocol: '服务器返回异常',
  'too-large': '文件过大',
  other: '操作失败',
};

/**
 * SFTP failures arrive as "<token>: <operation> failed at '<path>': <detail>".
 * Translating the token here keeps the classification out of every call site.
 */
function prettifySftpMessage(message: string): string {
  const match =
    /^(not-found|permission-denied|connection|protocol|too-large|other): (.+?) failed at '(.+?)': ([\s\S]*)$/.exec(
      message,
    );
  if (!match) {
    return message;
  }

  const [, token, operation, path, detail] = match;
  const label = SFTP_KIND_LABELS[token] ?? '操作失败';
  return `${label}（${path}）：${operation} — ${detail}`;
}

export function toAppError(err: unknown): AppError {
  if (err !== null && typeof err === 'object') {
    const e = err as Record<string, unknown>;
    const kind: AppErrorKind =
      typeof e.kind === 'string' ? (KIND_MAP[e.kind] ?? 'Unknown') : 'Unknown';
    const raw =
      typeof e.message === 'string' ? sanitizeMessage(e.message) : String(err);
    const message = kind === 'Sftp' ? prettifySftpMessage(raw) : raw;
    return { kind, message };
  }
  if (typeof err === 'string') {
    return { kind: 'Unknown', message: sanitizeMessage(err) };
  }
  return { kind: 'Unknown', message: 'An unknown error occurred' };
}

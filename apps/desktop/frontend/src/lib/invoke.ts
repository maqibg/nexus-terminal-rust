/**
 * Tauri invoke wrapper — replaces apiClient.ts (Axios).
 * All backend calls go through Tauri IPC instead of HTTP.
 */
import { invoke } from '@tauri-apps/api/core';
import { type AppErrorKind, toAppError } from './errors';

/** Backend AppError shape: #[serde(tag = "kind", content = "message")] */
export interface AppError {
  kind: string;
  message?: string;
}

/**
 * Structured error from a Tauri IPC call.
 * Carries `kind` for type-safe error discrimination:
 *   catch (e) { if (e instanceof TauriError && e.kind === 'Auth') { ... } }
 */
export class TauriError extends Error {
  readonly kind: AppErrorKind;
  constructor(appErr: { kind: AppErrorKind; message: string }) {
    super(appErr.message);
    this.name = 'TauriError';
    this.kind = appErr.kind;
  }
}

/**
 * Type-safe invoke with unified error handling.
 * Errors are converted via toAppError (which also sanitizes sensitive data).
 * Throws TauriError so callers can discriminate on `kind`.
 */
export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err: unknown) {
    throw new TauriError(toAppError(err));
  }
}

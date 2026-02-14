/**
 * Tauri invoke wrapper — replaces apiClient.ts (Axios).
 * All backend calls go through Tauri IPC instead of HTTP.
 */
import { invoke } from '@tauri-apps/api/core';

/** Backend AppError shape: #[serde(tag = "kind", content = "message")] */
export interface AppError {
  kind: string;
  message?: string;
}

/**
 * Type-safe invoke with unified error handling.
 */
export async function tauriInvoke<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(cmd, args);
  } catch (err: unknown) {
    const e = err as AppError;
    throw new Error(e?.message ?? e?.kind ?? String(err));
  }
}

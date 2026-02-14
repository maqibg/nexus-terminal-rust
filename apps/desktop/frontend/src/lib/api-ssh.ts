/**
 * SSH session API — replaces WebSocket-based SSH transport.
 * Uses Tauri invoke for commands + Tauri event listener for output streaming.
 */
import { tauriInvoke } from './invoke';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface SshSession {
  session_id: string;
  connection_id: number;
  connection_name: string;
}

export const sshApi = {
  connect: (connectionId: number, cols?: number, rows?: number) =>
    tauriInvoke<string>('ssh_connect', {
      req: { connection_id: connectionId, cols, rows },
    }),

  write: (sessionId: string, data: string) =>
    tauriInvoke<void>('ssh_write', {
      req: { session_id: sessionId, data },
    }),

  resize: (sessionId: string, cols: number, rows: number) =>
    tauriInvoke<void>('ssh_resize', {
      req: { session_id: sessionId, cols, rows },
    }),

  close: (sessionId: string) =>
    tauriInvoke<void>('ssh_close', {
      req: { session_id: sessionId },
    }),

  list: () => tauriInvoke<SshSession[]>('ssh_session_list'),
};

/**
 * Listen to SSH output events for a session.
 * Returns an unlisten function to stop listening.
 *
 * Events:
 *   ssh:output:{sessionId}  — stdout (base64)
 *   ssh:stderr:{sessionId}  — stderr (base64)
 *   ssh:exit:{sessionId}    — exit code
 *   ssh:close:{sessionId}   — channel closed
 */
export async function onSshOutput(
  sessionId: string,
  handlers: {
    onData?: (base64: string) => void;
    onStderr?: (base64: string) => void;
    onExit?: (code: number) => void;
    onClose?: () => void;
  },
): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];

  if (handlers.onData) {
    unlisteners.push(
      await listen<string>(`ssh:output:${sessionId}`, (e) => handlers.onData!(e.payload)),
    );
  }
  if (handlers.onStderr) {
    unlisteners.push(
      await listen<string>(`ssh:stderr:${sessionId}`, (e) => handlers.onStderr!(e.payload)),
    );
  }
  if (handlers.onExit) {
    unlisteners.push(
      await listen<number>(`ssh:exit:${sessionId}`, (e) => handlers.onExit!(e.payload)),
    );
  }
  if (handlers.onClose) {
    unlisteners.push(
      await listen<void>(`ssh:close:${sessionId}`, () => handlers.onClose!()),
    );
  }

  return () => unlisteners.forEach((fn) => fn());
}

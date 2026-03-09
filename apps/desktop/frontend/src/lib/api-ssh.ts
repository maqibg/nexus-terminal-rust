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

export interface SshExecResult {
  stdout: string;
  stderr: string;
  exit_code: number;
}

export interface SshOutputChunk {
  seq: number;
  stream: 'stdout' | 'stderr' | string;
  data: string;
}

export interface HostKeyEntry {
  host: string;
  port: number;
  fingerprint: string;
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

  executeCommand: (sessionId: string, command: string, timeoutMs?: number) =>
    tauriInvoke<SshExecResult>('ssh_exec_command', {
      req: { session_id: sessionId, command, timeout_ms: timeoutMs },
    }),

  takeOutputBacklog: (sessionId: string) =>
    tauriInvoke<SshOutputChunk[]>('ssh_take_output_backlog', {
      req: { session_id: sessionId },
    }),

  list: () => tauriInvoke<SshSession[]>('ssh_session_list'),

  // Host key management
  acceptHostKey: (host: string, port: number, fingerprint: string) =>
    tauriInvoke<void>('ssh_accept_host_key', { host, port, fingerprint }),

  hostKeyList: () => tauriInvoke<HostKeyEntry[]>('ssh_host_key_list'),

  hostKeyGet: (host: string, port: number) =>
    tauriInvoke<string | null>('ssh_host_key_get', { host, port }),

  hostKeyDelete: (host: string, port: number) =>
    tauriInvoke<boolean>('ssh_host_key_delete', { host, port }),
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
    onData?: (base64: string, chunk?: SshOutputChunk) => void;
    onStderr?: (base64: string, chunk?: SshOutputChunk) => void;
    onExit?: (code: number) => void;
    onClose?: () => void;
  },
): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];
  const normalizeChunk = (payload: string | SshOutputChunk, stream: 'stdout' | 'stderr'): SshOutputChunk => {
    if (typeof payload === 'string') {
      return {
        seq: -1,
        stream,
        data: payload,
      };
    }
    return payload;
  };

  if (handlers.onData) {
    unlisteners.push(
      await listen<string | SshOutputChunk>(`ssh:output:${sessionId}`, (e) => {
        const chunk = normalizeChunk(e.payload, 'stdout');
        handlers.onData!(chunk.data, chunk);
      }),
    );
  }
  if (handlers.onStderr) {
    unlisteners.push(
      await listen<string | SshOutputChunk>(`ssh:stderr:${sessionId}`, (e) => {
        const chunk = normalizeChunk(e.payload, 'stderr');
        handlers.onStderr!(chunk.data, chunk);
      }),
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

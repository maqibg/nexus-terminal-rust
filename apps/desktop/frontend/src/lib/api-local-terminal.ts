import { tauriInvoke } from './invoke';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface LocalTerminalSessionInfo {
  sessionId: string;
  shell: string;
}

export interface LocalTerminalOutputChunk {
  seq: number;
  stream: 'stdout' | 'stderr' | string;
  data: string;
}

export const localTerminalApi = {
  open: (shell?: string) =>
    tauriInvoke<string>('local_terminal_open', { req: { shell } }),

  write: (sessionId: string, dataBase64: string) =>
    tauriInvoke<void>('local_terminal_write', { req: { session_id: sessionId, data: dataBase64 } }),

  resize: (sessionId: string, cols: number, rows: number) =>
    tauriInvoke<void>('local_terminal_resize', { req: { session_id: sessionId, cols, rows } }),

  close: (sessionId: string) =>
    tauriInvoke<boolean>('local_terminal_close', { req: { session_id: sessionId } }),

  list: () => tauriInvoke<LocalTerminalSessionInfo[]>('local_terminal_session_list'),

  takeOutputBacklog: (sessionId: string) =>
    tauriInvoke<LocalTerminalOutputChunk[]>('local_terminal_take_output_backlog', { req: { session_id: sessionId } }),
};

export async function onLocalTerminalOutput(
  sessionId: string,
  handlers: {
    onData?: (base64: string, chunk?: LocalTerminalOutputChunk) => void;
    onStderr?: (base64: string, chunk?: LocalTerminalOutputChunk) => void;
    onExit?: (code: number) => void;
    onClose?: () => void;
  },
): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];

  if (handlers.onData) {
    unlisteners.push(
      await listen<LocalTerminalOutputChunk>(`local:output:${sessionId}`, (e) => {
        handlers.onData?.(e.payload.data, e.payload);
      }),
    );
  }
  if (handlers.onStderr) {
    unlisteners.push(
      await listen<LocalTerminalOutputChunk>(`local:stderr:${sessionId}`, (e) => {
        handlers.onStderr?.(e.payload.data, e.payload);
      }),
    );
  }
  if (handlers.onExit) {
    unlisteners.push(
      await listen<number>(`local:exit:${sessionId}`, (e) => {
        handlers.onExit?.(e.payload);
      }),
    );
  }
  if (handlers.onClose) {
    unlisteners.push(
      await listen<void>(`local:close:${sessionId}`, () => {
        handlers.onClose?.();
      }),
    );
  }

  return () => unlisteners.forEach(fn => fn());
}


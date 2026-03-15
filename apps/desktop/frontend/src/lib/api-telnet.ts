import { tauriInvoke } from './invoke';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export interface TelnetSessionInfo {
  sessionId: string;
  host: string;
  port: number;
}

export interface TelnetOutputChunk {
  seq: number;
  data: string;
}

export const telnetApi = {
  connect: (host: string, port: number) =>
    tauriInvoke<string>('telnet_connect', { req: { host, port } }),

  write: (sessionId: string, dataBase64: string) =>
    tauriInvoke<void>('telnet_write', { req: { session_id: sessionId, data: dataBase64 } }),

  close: (sessionId: string) =>
    tauriInvoke<boolean>('telnet_close', { req: { session_id: sessionId } }),

  list: () => tauriInvoke<TelnetSessionInfo[]>('telnet_session_list'),

  takeOutputBacklog: (sessionId: string) =>
    tauriInvoke<TelnetOutputChunk[]>('telnet_take_output_backlog', { req: { session_id: sessionId } }),
};

export async function onTelnetOutput(
  sessionId: string,
  handlers: {
    onData?: (base64: string, chunk?: TelnetOutputChunk) => void;
    onClose?: () => void;
  },
): Promise<UnlistenFn> {
  const unlisteners: UnlistenFn[] = [];

  if (handlers.onData) {
    unlisteners.push(
      await listen<TelnetOutputChunk>(`telnet:output:${sessionId}`, (e) => {
        handlers.onData?.(e.payload.data, e.payload);
      }),
    );
  }

  if (handlers.onClose) {
    unlisteners.push(
      await listen<void>(`telnet:close:${sessionId}`, () => {
        handlers.onClose?.();
      }),
    );
  }

  return () => unlisteners.forEach(fn => fn());
}


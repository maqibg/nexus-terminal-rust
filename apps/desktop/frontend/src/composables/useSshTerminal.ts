import { ref, type Ref, onUnmounted } from 'vue';
import { Terminal } from '@xterm/xterm';
import { sshApi, onSshOutput } from '@/lib/api';
import type { UnlistenFn } from '@tauri-apps/api/event';

/**
 * SSH terminal lifecycle composable.
 * Manages xterm.js Terminal + Tauri SSH session.
 */
export function useSshTerminal(sessionId: Ref<string>) {
  const terminal = ref<Terminal | null>(null);
  const connected = ref(false);
  let unlisten: UnlistenFn | null = null;

  async function connect(connectionId: number, cols = 80, rows = 24): Promise<string> {
    const sid = await sshApi.connect(connectionId, cols, rows);
    sessionId.value = sid;
    connected.value = true;
    await setupListeners(sid);
    return sid;
  }

  async function setupListeners(sid: string) {
    unlisten = await onSshOutput(sid, {
      onData(base64) {
        const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
        terminal.value?.write(bytes);
      },
      onStderr(base64) {
        const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
        terminal.value?.write(bytes);
      },
      onExit() {
        connected.value = false;
      },
      onClose() {
        connected.value = false;
      },
    });
  }

  function write(data: string) {
    if (sessionId.value) {
      const base64 = btoa(data);
      sshApi.write(sessionId.value, base64);
    }
  }

  function resize(cols: number, rows: number) {
    if (sessionId.value) {
      sshApi.resize(sessionId.value, cols, rows);
    }
  }

  async function disconnect() {
    if (sessionId.value) {
      await sshApi.close(sessionId.value);
    }
    cleanup();
  }

  function cleanup() {
    unlisten?.();
    unlisten = null;
    connected.value = false;
  }

  onUnmounted(cleanup);

  return { terminal, connected, connect, write, resize, disconnect, setupListeners };
}

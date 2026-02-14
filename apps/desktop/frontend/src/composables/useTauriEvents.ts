import { onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

/**
 * Tauri Event listen/unlisten lifecycle wrapper.
 * Auto-cleans all listeners on component unmount.
 */
export function useTauriEvents() {
  const cleanups: UnlistenFn[] = [];

  async function on<T>(event: string, handler: (payload: T) => void): Promise<UnlistenFn> {
    const unlisten = await listen<T>(event, (e) => handler(e.payload));
    cleanups.push(unlisten);
    return unlisten;
  }

  async function onSession<T>(
    sessionId: string,
    event: string,
    handler: (payload: T) => void,
  ): Promise<UnlistenFn> {
    return on<T>(`${event}:${sessionId}`, handler);
  }

  function cleanup() {
    cleanups.forEach((fn) => fn());
    cleanups.length = 0;
  }

  onUnmounted(cleanup);

  return { on, onSession, cleanup };
}

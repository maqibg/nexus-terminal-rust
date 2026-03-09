import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { v4 as uuidv4 } from 'uuid';

export type SessionProtocol = 'SSH' | 'RDP' | 'VNC';

export interface SessionInfo {
  id: string;
  connectionId: number;
  connectionName: string;
  protocol: SessionProtocol;
  status: 'connecting' | 'connected' | 'disconnected';
  createdAt: string;
  sftpReady: boolean;
  sftpSessionId: string | null;
  currentPath: string;
  desktopSessionId?: string | null;
  vncWsPort?: number | null;
  vncPassword?: string | null;
}

export const useSessionStore = defineStore('session', () => {
  const sessions = ref<Map<string, SessionInfo>>(new Map());
  const activeSessionId = ref<string | null>(null);

  const activeSession = computed(() =>
    activeSessionId.value ? sessions.value.get(activeSessionId.value) : undefined
  );

  const sessionList = computed(() => Array.from(sessions.value.values()));

  function addSession(info: SessionInfo) {
    const next = new Map(sessions.value);
    next.set(info.id, info);
    sessions.value = next;
    if (!activeSessionId.value) activeSessionId.value = info.id;
  }

  function removeSession(id: string) {
    const next = new Map(sessions.value);
    next.delete(id);
    sessions.value = next;

    if (activeSessionId.value === id) {
      const remaining = Array.from(next.keys());
      activeSessionId.value = remaining.length > 0 ? remaining[remaining.length - 1] : null;
    }
  }

  function setActive(id: string) {
    if (sessions.value.has(id)) activeSessionId.value = id;
  }

  function getSession(id: string) {
    return sessions.value.get(id);
  }

  function updateStatus(id: string, status: SessionInfo['status']) {
    const current = sessions.value.get(id);
    if (!current) return;

    const next = new Map(sessions.value);
    next.set(id, { ...current, status });
    sessions.value = next;
  }

  function setSftpReady(id: string, ready: boolean) {
    const current = sessions.value.get(id);
    if (!current) return;

    const next = new Map(sessions.value);
    next.set(id, { ...current, sftpReady: ready });
    sessions.value = next;
  }

  function setSftpSession(id: string, sftpSessionId: string | null) {
    const current = sessions.value.get(id);
    if (!current) return;

    const next = new Map(sessions.value);
    next.set(id, {
      ...current,
      sftpSessionId,
      sftpReady: !!sftpSessionId,
    });
    sessions.value = next;
  }

  function setCurrentPath(id: string, path: string) {
    const current = sessions.value.get(id);
    if (!current) return;

    const next = new Map(sessions.value);
    next.set(id, { ...current, currentPath: path });
    sessions.value = next;
  }

  function createSession(connectionId: number, connectionName: string, protocol: SessionProtocol = 'SSH'): string {
    const id = `session-${uuidv4()}`;
    addSession({
      id,
      connectionId,
      connectionName,
      protocol,
      status: 'connecting',
      createdAt: new Date().toISOString(),
      sftpReady: false,
      sftpSessionId: null,
      currentPath: '/',
      desktopSessionId: null,
      vncWsPort: null,
      vncPassword: null,
    });
    return id;
  }

  function createVncSession(connectionId: number, connectionName: string, desktopSessionId: string, wsPort: number, password?: string | null): string {
    const id = `vnc-${uuidv4()}`;
    addSession({
      id,
      connectionId,
      connectionName,
      protocol: 'VNC',
      status: 'connected',
      createdAt: new Date().toISOString(),
      sftpReady: false,
      sftpSessionId: null,
      currentPath: '/',
      desktopSessionId,
      vncWsPort: wsPort,
      vncPassword: password ?? null,
    });
    return id;
  }

  return {
    sessions,
    activeSessionId,
    activeSession,
    sessionList,
    addSession,
    removeSession,
    setActive,
    getSession,
    updateStatus,
    setSftpReady,
    setSftpSession,
    setCurrentPath,
    createSession,
    createVncSession,
  };
});
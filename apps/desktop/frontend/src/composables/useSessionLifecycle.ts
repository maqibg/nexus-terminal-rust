import { desktopApi, sftpApi, sshApi, type Connection } from '@/lib/api';
import { toAppError } from '@/lib/errors';
import { useSessionStore } from '@/stores/session';
import { useUiNotificationsStore } from '@/stores/uiNotifications';

type ErrorReporter = (title: string, message: string) => Promise<void> | void;

interface ConnectHooks {
  onSessionActivated?: (sessionId: string, protocol: 'SSH' | 'VNC') => Promise<void> | void;
}

export function useSessionLifecycle(reportError?: ErrorReporter) {
  const sessionStore = useSessionStore();
  const notifications = useUiNotificationsStore();

  const emitError: ErrorReporter = reportError ?? ((title, message) => {
    notifications.addNotification({ type: 'error', message: `${title}: ${message}` });
  });

  async function warmupSftp(sessionId: string, connectionId: number) {
    try {
      const sftpSessionId = await sftpApi.open(connectionId);
      sessionStore.setSftpSession(sessionId, sftpSessionId);
    } catch {
      // allow terminal-only usage when SFTP warmup fails
    }
  }

  async function connectConnection(conn: Connection, hooks?: ConnectHooks): Promise<string | null> {
    const connType = String(conn.type ?? 'SSH').toUpperCase();

    if (connType === 'RDP') {
      try {
        await desktopApi.openRdpConnection(conn.id);
      } catch (error) {
        await emitError('RDP 启动失败', toAppError(error).message);
      }
      return null;
    }

    if (connType === 'VNC') {
      try {
        const vncSession = await desktopApi.openVncConnection(conn.id);
        const localSessionId = sessionStore.createVncSession(
          conn.id,
          conn.name,
          vncSession.session_id,
          vncSession.ws_port,
          vncSession.password,
        );
        sessionStore.setActive(localSessionId);
        await hooks?.onSessionActivated?.(localSessionId, 'VNC');
        return localSessionId;
      } catch (error) {
        await emitError('VNC 启动失败', toAppError(error).message);
        return null;
      }
    }

    const pendingSessionId = sessionStore.createSession(conn.id, conn.name, 'SSH');
    sessionStore.setActive(pendingSessionId);
    await hooks?.onSessionActivated?.(pendingSessionId, 'SSH');

    try {
      const realSessionId = await sshApi.connect(conn.id);
      sessionStore.removeSession(pendingSessionId);
      sessionStore.addSession({
        id: realSessionId,
        connectionId: conn.id,
        connectionName: conn.name,
        protocol: 'SSH',
        status: 'connected',
        createdAt: new Date().toISOString(),
        sftpReady: false,
        sftpSessionId: null,
        currentPath: '/',
        desktopSessionId: null,
        vncWsPort: null,
        vncPassword: null,
      });
      sessionStore.setActive(realSessionId);
      void warmupSftp(realSessionId, conn.id);
      return realSessionId;
    } catch (error) {
      sessionStore.updateStatus(pendingSessionId, 'disconnected');
      await emitError('SSH 连接失败', toAppError(error).message);
      return null;
    }
  }

  async function closeSession(sessionId: string): Promise<void> {
    const session = sessionStore.getSession(sessionId);
    if (!session) {
      return;
    }

    if (session.protocol === 'VNC') {
      if (session.desktopSessionId) {
        try {
          await desktopApi.disconnectVncSession(session.desktopSessionId);
        } catch {
          // ignore best-effort close failures
        }
      }
      sessionStore.removeSession(sessionId);
      return;
    }

    if (session.sftpSessionId) {
      try {
        await sftpApi.close(session.sftpSessionId);
      } catch {
        // ignore best-effort SFTP close failures
      }
    }

    try {
      await sshApi.close(sessionId);
    } catch {
      // ignore best-effort backend close failures
    }
    sessionStore.removeSession(sessionId);
  }

  return {
    connectConnection,
    closeSession,
  };
}

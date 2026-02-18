import { tauriInvoke } from './invoke';

export interface OpenRdpPayload {
  host: string;
  port?: number;
  username?: string;
  password?: string;
  options?: Record<string, unknown>;
  connection_id?: number;
  connection_name?: string;
}

export interface OpenVncPayload {
  host: string;
  port?: number;
  password?: string;
  options?: Record<string, unknown>;
  connection_id?: number;
  connection_name?: string;
}

export interface RdpSessionStatus {
  connection_id: number;
  connection_name?: string | null;
  host: string;
  port: number;
  status: 'connecting' | 'connected' | 'disconnected' | 'error';
  process_id?: number | null;
  started_at_ms: number;
  last_error?: string | null;
}

export interface VncSessionInfo {
  session_id: string;
  connection_id?: number | null;
  connection_name?: string | null;
  host: string;
  port: number;
  ws_port: number;
  status: 'connecting' | 'connected' | 'disconnected' | 'error';
  created_at_ms: number;
  last_error?: string | null;
  password?: string | null;
  options?: Record<string, unknown>;
}

export const desktopApi = {
  openRdp: (payload: OpenRdpPayload) =>
    tauriInvoke<void>('desktop_open_rdp', {
      req: {
        host: payload.host,
        port: payload.port,
        username: payload.username,
        password: payload.password,
        options: payload.options,
        connection_id: payload.connection_id,
        connectionId: payload.connection_id,
        connection_name: payload.connection_name,
        connectionName: payload.connection_name,
      },
    }),

  openRdpConnection: (connectionId: number) =>
    tauriInvoke<void>('desktop_open_rdp_connection', {
      connectionId,
    }),

  getRdpStatus: (connectionId: number) =>
    tauriInvoke<RdpSessionStatus | null>('desktop_rdp_status', {
      connectionId,
    }),

  listRdpSessions: () =>
    tauriInvoke<RdpSessionStatus[]>('desktop_rdp_list_sessions'),

  disconnectRdpConnection: (connectionId: number) =>
    tauriInvoke<boolean>('desktop_rdp_disconnect_connection', {
      connectionId,
    }),

  openVnc: (payload: OpenVncPayload) =>
    tauriInvoke<VncSessionInfo>('desktop_open_vnc', {
      req: {
        host: payload.host,
        port: payload.port,
        password: payload.password,
        options: payload.options,
        connection_id: payload.connection_id,
        connectionId: payload.connection_id,
        connection_name: payload.connection_name,
        connectionName: payload.connection_name,
      },
    }),

  openVncConnection: (connectionId: number) =>
    tauriInvoke<VncSessionInfo>('desktop_open_vnc_connection', {
      connectionId,
    }),

  disconnectVncSession: (sessionId: string) =>
    tauriInvoke<boolean>('desktop_vnc_disconnect', {
      sessionId,
    }),

  getVncSessionStatus: (sessionId: string) =>
    tauriInvoke<VncSessionInfo | null>('desktop_vnc_status', {
      sessionId,
    }),

  listVncSessions: () =>
    tauriInvoke<VncSessionInfo[]>('desktop_vnc_list_sessions'),
};
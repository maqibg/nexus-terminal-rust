import { tauriInvoke } from './invoke';

export interface OpenRdpPayload {
  host: string;
  port?: number;
  username?: string;
  password?: string;
}

export const desktopApi = {
  openRdp: (payload: OpenRdpPayload) =>
    tauriInvoke<void>('desktop_open_rdp', {
      req: {
        host: payload.host,
        port: payload.port,
        username: payload.username,
        password: payload.password,
      },
    }),

  openRdpConnection: (connectionId: number) =>
    tauriInvoke<void>('desktop_open_rdp_connection', {
      connection_id: connectionId,
    }),
};

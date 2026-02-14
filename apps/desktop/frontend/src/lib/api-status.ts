import { tauriInvoke } from './invoke';

export interface BackendHealth {
  status: string;
  version: string;
}

export const statusApi = {
  getBackendHealth: () => tauriInvoke<BackendHealth>('get_backend_health'),
};

import { tauriInvoke } from './invoke';

export interface BackendHealth {
  status: string;
  version: string;
}

export interface RuntimePaths {
  exeDir: string;
  dataDir: string;
  downloadDir: string;
  tempDir: string;
}

export const statusApi = {
  getBackendHealth: () => tauriInvoke<BackendHealth>('get_backend_health'),
  getRuntimePaths: () => tauriInvoke<RuntimePaths>('get_runtime_paths'),
};

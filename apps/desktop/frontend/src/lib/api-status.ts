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

export interface RuntimeStatusSnapshot {
  sessionId: string;
  connectionId: number;
  timestamp: number;
  ipAddress: string;
  cpuModel: string;
  osName: string;
  cpuPercent: number;
  cpuCores?: number | null;
  cpuPerCore: number[];
  memUsed: number;
  memTotal: number;
  memPercent: number;
  memFree: number;
  memBuffers: number;
  memCached: number;
  swapUsed: number;
  swapTotal: number;
  swapPercent: number;
  diskUsed: number;
  diskTotal: number;
  diskPercent: number;
  disks: DiskUsageEntry[];
  topProcesses: StatusProcessEntry[];
  netInterface: string;
  netRxTotal: number;
  netTxTotal: number;
  netRxRate: number;
  netTxRate: number;
  netInterfaces: NetInterfaceEntry[];
}

export interface DiskUsageEntry {
  name: string;
  usedKb: number;
  totalKb: number;
  percent: number;
}

export interface StatusProcessEntry {
  pid: string;
  memPercent: number;
  command: string;
}

export interface NetInterfaceEntry {
  name: string;
  rxTotal: number;
  txTotal: number;
  rxRate: number;
  txRate: number;
}

export const statusApi = {
  getBackendHealth: () => tauriInvoke<BackendHealth>('get_backend_health'),
  getRuntimePaths: () => tauriInvoke<RuntimePaths>('get_runtime_paths'),
  getConnectionRuntimeStatus: (args: { connectionId?: number; sessionId?: string }) =>
    tauriInvoke<RuntimeStatusSnapshot>('get_connection_runtime_status', {
      connectionId: args.connectionId,
      sessionId: args.sessionId,
    }),
  setStatusMonitorEnabled: (enabled: boolean) =>
    tauriInvoke<void>('set_status_monitor_enabled', { enabled }),
  subscribe: (sessionId: string, consumerId: string) =>
    tauriInvoke<void>('status_subscribe', { sessionId, consumerId }),
  unsubscribe: (sessionId: string, consumerId: string) =>
    tauriInvoke<void>('status_unsubscribe', { sessionId, consumerId }),
};

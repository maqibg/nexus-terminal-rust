import { computed, onUnmounted, readonly, ref, watch, type Ref } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { storeToRefs } from 'pinia';

import {
  statusApi,
  type DiskUsageEntry,
  type NetInterfaceEntry,
  type StatusProcessEntry,
} from '@/lib/api-status';
import { useSessionStore, type SessionProtocol, type SessionInfo } from '@/stores/session';
import { useSettingsStore } from '@/stores/settings';

const HISTORY_POINTS = 60;

export interface SessionStatusSnapshot {
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
  disks?: DiskUsageEntry[];
  topProcesses: StatusProcessEntry[];
  netInterface: string;
  netRxTotal: number;
  netTxTotal: number;
  netRxRate: number;
  netTxRate: number;
  netInterfaces: NetInterfaceEntry[];
}

export interface StatusErrorPayload {
  sessionId: string;
  message: string;
  timestamp?: number;
  degraded?: boolean;
  unsupported?: boolean;
}

export interface UseStatusMonitorOptions {
  sessionId?: Ref<string | null>;
  sessionStatus?: Ref<SessionInfo['status'] | undefined>;
  sessionProtocol?: Ref<SessionProtocol | undefined>;
  active?: Ref<boolean>;
  monitorEnabled?: Ref<boolean>;
  consumerId?: string;
}

function createEmptyHistory(): (number | null)[] {
  return Array(HISTORY_POINTS).fill(null);
}

function pushHistory(history: (number | null)[], value: number): (number | null)[] {
  const next = [...history];
  next.shift();
  next.push(Number.isFinite(value) ? value : null);
  return next;
}

export function useStatusMonitor(options: UseStatusMonitorOptions = {}) {
  const sessionStore = useSessionStore();
  const settingsStore = useSettingsStore();
  const { activeSessionId, activeSession } = storeToRefs(sessionStore);

  const sessionIdRef = options.sessionId ?? activeSessionId;
  const sessionStatusRef = options.sessionStatus ?? computed(() => activeSession.value?.status);
  const sessionProtocolRef = options.sessionProtocol ?? computed(() => activeSession.value?.protocol);
  const activeRef = options.active ?? computed(() => true);
  const monitorEnabled = options.monitorEnabled ?? computed(() => settingsStore.getBoolean('statusMonitorEnabled', true));
  const consumerId = options.consumerId ?? 'workspace-pane';
  const monitorConfigKey = computed(
    () =>
      `${settingsStore.getInteger('statusMonitorIntervalSeconds', 3, 1)}:${
        settingsStore.getBoolean('statusMonitorFailureBackoffEnabled', true) ? '1' : '0'
      }`,
  );

  const currentStatus = ref<SessionStatusSnapshot | null>(null);
  const statusError = ref<string | null>(null);
  const lastUpdatedAt = ref<number | null>(null);
  const waitingForFirstSample = ref(false);
  const cpuHistory = ref<(number | null)[]>(createEmptyHistory());
  const memUsedHistory = ref<(number | null)[]>(createEmptyHistory());
  const netRxHistory = ref<(number | null)[]>(createEmptyHistory());
  const netTxHistory = ref<(number | null)[]>(createEmptyHistory());

  let unlistenUpdate: UnlistenFn | null = null;
  let unlistenError: UnlistenFn | null = null;
  let bindVersion = 0;
  let subscribedSessionId: string | null = null;

  function resetState() {
    currentStatus.value = null;
    statusError.value = null;
    lastUpdatedAt.value = null;
    waitingForFirstSample.value = false;
    cpuHistory.value = createEmptyHistory();
    memUsedHistory.value = createEmptyHistory();
    netRxHistory.value = createEmptyHistory();
    netTxHistory.value = createEmptyHistory();
  }

  function clearListeners() {
    unlistenUpdate?.();
    unlistenError?.();
    unlistenUpdate = null;
    unlistenError = null;
  }

  function applySnapshot(snapshot: SessionStatusSnapshot) {
    currentStatus.value = snapshot;
    statusError.value = null;
    lastUpdatedAt.value = snapshot.timestamp ?? Date.now();
    waitingForFirstSample.value = false;
    cpuHistory.value = pushHistory(cpuHistory.value, snapshot.cpuPercent ?? 0);
    memUsedHistory.value = pushHistory(memUsedHistory.value, snapshot.memUsed ?? 0);
    netRxHistory.value = pushHistory(netRxHistory.value, snapshot.netRxRate ?? 0);
    netTxHistory.value = pushHistory(netTxHistory.value, snapshot.netTxRate ?? 0);
  }

  async function releaseSubscription() {
    const sessionId = subscribedSessionId;
    subscribedSessionId = null;
    if (!sessionId) {
      return;
    }
    try {
      await statusApi.unsubscribe(sessionId, consumerId);
    } catch {
      // Ignore unsubscribe failures during session switches and teardown.
    }
  }

  async function bindSession(sessionId: string | null) {
    bindVersion += 1;
    const version = bindVersion;

    await releaseSubscription();
    clearListeners();
    resetState();

    if (!sessionId) {
      return;
    }

    waitingForFirstSample.value = true;

    try {
      await statusApi.subscribe(sessionId, consumerId);
      if (bindVersion !== version) {
        await statusApi.unsubscribe(sessionId, consumerId).catch(() => undefined);
        return;
      }
      subscribedSessionId = sessionId;
    } catch (error) {
      statusError.value = normalizeError(error, '状态采集订阅失败');
      waitingForFirstSample.value = false;
      return;
    }

    try {
      const snapshot = await statusApi.getConnectionRuntimeStatus({ sessionId });
      if (bindVersion === version) {
        applySnapshot(snapshot);
      }
    } catch {
      // Fall back to the event-driven first sample.
    }

    const updateUnlisten = await listen<SessionStatusSnapshot>(`status:update:${sessionId}`, (event) => {
      if (bindVersion === version) {
        applySnapshot(event.payload);
      }
    });
    if (bindVersion !== version) {
      updateUnlisten();
      return;
    }
    unlistenUpdate = updateUnlisten;

    const errorUnlisten = await listen<StatusErrorPayload>(`status:error:${sessionId}`, (event) => {
      if (bindVersion !== version) {
        return;
      }
      statusError.value = event.payload?.message?.trim() || '状态采集失败';
      waitingForFirstSample.value = false;
    });
    if (bindVersion !== version) {
      errorUnlisten();
      return;
    }
    unlistenError = errorUnlisten;
  }

  watch(
    [
      sessionIdRef,
      sessionStatusRef,
      sessionProtocolRef,
      monitorEnabled,
      activeRef,
      monitorConfigKey,
    ],
    ([sessionId, status, protocol, enabled, active]) => {
      if (enabled && active && status === 'connected' && sessionId && protocol === 'SSH') {
        void bindSession(sessionId);
      } else {
        void bindSession(null);
      }
    },
    { immediate: true },
  );

  onUnmounted(() => {
    bindVersion += 1;
    clearListeners();
    void releaseSubscription();
  });

  const isWaitingForFirstSample = computed(
    () => waitingForFirstSample.value && !currentStatus.value && !statusError.value,
  );

  return {
    currentStatus: readonly(currentStatus),
    statusError: readonly(statusError),
    lastUpdatedAt: readonly(lastUpdatedAt),
    isWaitingForFirstSample,
    cpuHistory: readonly(cpuHistory),
    memUsedHistory: readonly(memUsedHistory),
    netRxHistory: readonly(netRxHistory),
    netTxHistory: readonly(netTxHistory),
  };
}

function normalizeError(error: unknown, fallback: string): string {
  if (error instanceof Error && error.message.trim()) {
    return error.message;
  }
  if (typeof error === 'string' && error.trim()) {
    return error;
  }
  return fallback;
}

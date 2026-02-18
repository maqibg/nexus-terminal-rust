import { computed, onUnmounted, readonly, ref, watch } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { storeToRefs } from 'pinia';

import { useSessionStore } from '@/stores/session';

const HISTORY_POINTS = 60;

export interface SessionStatusSnapshot {
  sessionId: string;
  connectionId: number;
  timestamp: number;
  ipAddress: string;
  cpuModel: string;
  osName: string;
  cpuPercent: number;
  memUsed: number;
  memTotal: number;
  memPercent: number;
  swapUsed: number;
  swapTotal: number;
  swapPercent: number;
  diskUsed: number;
  diskTotal: number;
  diskPercent: number;
  netInterface: string;
  netRxTotal: number;
  netTxTotal: number;
  netRxRate: number;
  netTxRate: number;
}

interface StatusErrorPayload {
  sessionId?: string;
  message?: string;
  timestamp?: number;
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

export function useStatusMonitor() {
  const sessionStore = useSessionStore();
  const { activeSessionId, activeSession } = storeToRefs(sessionStore);

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

  async function bindSession(sessionId: string | null) {
    bindVersion += 1;
    const version = bindVersion;

    clearListeners();
    resetState();

    if (!sessionId) {
      return;
    }

    waitingForFirstSample.value = true;

    const updateUnlisten = await listen<SessionStatusSnapshot>(`status:update:${sessionId}`, (event) => {
      if (bindVersion !== version) {
        return;
      }

      const payload = event.payload;
      currentStatus.value = payload;
      statusError.value = null;
      lastUpdatedAt.value = payload.timestamp ?? Date.now();
      waitingForFirstSample.value = false;

      cpuHistory.value = pushHistory(cpuHistory.value, payload.cpuPercent ?? 0);
      memUsedHistory.value = pushHistory(memUsedHistory.value, payload.memUsed ?? 0);
      netRxHistory.value = pushHistory(netRxHistory.value, payload.netRxRate ?? 0);
      netTxHistory.value = pushHistory(netTxHistory.value, payload.netTxRate ?? 0);
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

      const message = event.payload?.message?.trim() || '状态采集失败';
      statusError.value = message;
      waitingForFirstSample.value = false;
    });

    if (bindVersion !== version) {
      errorUnlisten();
      return;
    }
    unlistenError = errorUnlisten;
  }

  watch(
    [activeSessionId, () => activeSession.value?.status],
    ([sessionId, status]) => {
      if (status === 'connected' && sessionId && activeSession.value?.protocol === 'SSH') {
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

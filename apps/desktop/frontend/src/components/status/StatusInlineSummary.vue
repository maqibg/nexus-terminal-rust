<template>
  <div v-if="inlineEnabled && isSshSession" class="inline-monitor" :class="{ 'inline-monitor-open': detailsOpen }">
    <div class="summary-row">
      <button class="summary-pill" type="button" @click="toggleDetails">
        <span class="pill-item"><i class="fas fa-microchip"></i><span>{{ cpuLabel }}</span></span>
        <span class="pill-item"><i class="fas fa-memory"></i><span>{{ memLabel }}</span></span>
        <span class="pill-item"><i class="fas fa-hard-drive"></i><span>{{ diskLabel }}</span></span>
        <span class="pill-item network-down"><i class="fas fa-arrow-down"></i><span>{{ downLabel }}</span></span>
        <span class="pill-item network-up"><i class="fas fa-arrow-up"></i><span>{{ upLabel }}</span></span>
      </button>
      <button class="summary-action" type="button" title="高亮状态面板" @click="focusPanel"><i class="fas fa-chart-line"></i></button>
    </div>

    <div v-if="detailsOpen" class="details-panel">
      <div v-if="statusError" class="details-banner">{{ statusError }}</div>
      <div v-else-if="isWaitingForFirstSample" class="details-banner details-banner-info">状态采集中...</div>
      <StatusCpuGrid v-if="showPerCoreCpu && currentStatus?.cpuPerCore?.length" :usages="currentStatus.cpuPerCore" />
      <StatusDiskList v-if="currentStatus?.disks?.length" :disks="currentStatus.disks" />
      <StatusNetworkList v-if="showInterfaceDetails && currentStatus?.netInterfaces?.length" :interfaces="currentStatus.netInterfaces" />
      <StatusProcessList v-if="showTopProcesses && currentStatus?.topProcesses?.length" :processes="currentStatus.topProcesses" />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';

import StatusCpuGrid from '@/components/status/StatusCpuGrid.vue';
import StatusDiskList from '@/components/status/StatusDiskList.vue';
import StatusNetworkList from '@/components/status/StatusNetworkList.vue';
import StatusProcessList from '@/components/status/StatusProcessList.vue';
import { useStatusMonitor } from '@/composables/useStatusMonitor';
import { useSessionStore } from '@/stores/session';
import { useSettingsStore } from '@/stores/settings';

const props = defineProps<{
  sessionId: string;
  active: boolean;
}>();

const sessionStore = useSessionStore();
const settingsStore = useSettingsStore();
const detailsOpen = ref(false);

const currentSession = computed(() => sessionStore.getSession(props.sessionId));
const isSshSession = computed(() => currentSession.value?.protocol === 'SSH');
const inlineEnabled = computed(
  () =>
    settingsStore.getBoolean('statusMonitorEnabled', true)
    && settingsStore.getBoolean('statusMonitorInlineSummaryEnabled', false),
);
const showTopProcesses = computed(() => settingsStore.getBoolean('statusMonitorShowTopProcesses', true));
const showPerCoreCpu = computed(() => settingsStore.getBoolean('statusMonitorShowPerCoreCpu', true));
const showInterfaceDetails = computed(() => settingsStore.getBoolean('statusMonitorShowInterfaceDetails', true));

const {
  currentStatus,
  statusError,
  isWaitingForFirstSample,
} = useStatusMonitor({
  sessionId: computed(() => (isSshSession.value ? props.sessionId : null)),
  sessionStatus: computed(() => currentSession.value?.status),
  sessionProtocol: computed(() => currentSession.value?.protocol),
  active: computed(() => props.active),
  monitorEnabled: inlineEnabled,
  consumerId: `terminal-inline:${props.sessionId}`,
});

const cpuLabel = computed(() => {
  if (isWaitingForFirstSample.value) return '...';
  return currentStatus.value?.cpuPercent ? `${Math.round(currentStatus.value.cpuPercent)}%` : '--';
});
const memLabel = computed(() => {
  const used = currentStatus.value?.memUsed;
  const total = currentStatus.value?.memTotal;
  if (typeof used !== 'number' || typeof total !== 'number' || total <= 0) return '--';
  return `${(used / 1024).toFixed(1)}/${(total / 1024).toFixed(1)}G`;
});
const diskLabel = computed(() => {
  const percent = currentStatus.value?.diskPercent;
  return typeof percent === 'number' ? `${Math.round(percent)}%` : '--';
});
const downLabel = computed(() => formatRate(currentStatus.value?.netRxRate));
const upLabel = computed(() => formatRate(currentStatus.value?.netTxRate));

function toggleDetails() {
  detailsOpen.value = !detailsOpen.value;
}

function focusPanel() {
  window.dispatchEvent(new Event('nexus:status-monitor-focus'));
  detailsOpen.value = false;
}

function formatRate(bytes: number | undefined): string {
  if (!Number.isFinite(bytes)) return '--';
  const safe = Math.max(0, Number(bytes));
  if (safe < 1024) return `${safe.toFixed(0)} B/s`;
  if (safe < 1024 * 1024) return `${(safe / 1024).toFixed(1)} KB/s`;
  if (safe < 1024 * 1024 * 1024) return `${(safe / 1024 / 1024).toFixed(1)} MB/s`;
  return `${(safe / 1024 / 1024 / 1024).toFixed(1)} GB/s`;
}
</script>

<style scoped>
.inline-monitor {
  position: absolute;
  left: 14px;
  right: 18px;
  bottom: 12px;
  z-index: 4;
  display: flex;
  flex-direction: column;
  gap: 10px;
  pointer-events: none;
}
.summary-row { display: flex; align-items: center; gap: 8px; pointer-events: auto; }
.summary-pill {
  flex: 1;
  min-width: 0;
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
  border: 1px solid color-mix(in srgb, var(--border) 84%, transparent);
  border-radius: 999px;
  padding: 8px 12px;
  background: color-mix(in srgb, var(--bg-base) 82%, transparent);
  color: var(--text);
  cursor: pointer;
  transition: background-color 0.18s ease, border-color 0.18s ease;
  box-shadow: 0 10px 24px color-mix(in srgb, var(--bg-base) 72%, transparent);
}
.summary-pill:hover { border-color: rgba(34, 197, 94, 0.32); background: color-mix(in srgb, var(--bg-surface0) 86%, transparent); }
.summary-action {
  border: 1px solid color-mix(in srgb, var(--border) 84%, transparent);
  border-radius: 999px;
  width: 36px;
  height: 36px;
  background: color-mix(in srgb, var(--bg-base) 82%, transparent);
  color: #22c55e;
  cursor: pointer;
}
.pill-item { display: inline-flex; align-items: center; gap: 6px; font-size: calc(11px + var(--ui-font-size-offset)); white-space: nowrap; }
.network-down { color: #22c55e; }
.network-up { color: #38bdf8; }
.details-panel {
  pointer-events: auto;
  display: grid;
  gap: 10px;
  max-height: min(46vh, 380px);
  overflow: auto;
  padding: 12px;
  border: 1px solid color-mix(in srgb, var(--border) 84%, transparent);
  border-radius: 16px;
  background: color-mix(in srgb, var(--bg-base) 92%, transparent);
  box-shadow: 0 18px 34px color-mix(in srgb, var(--bg-base) 76%, transparent);
}
.details-banner { padding: 10px 12px; border-radius: 12px; background: rgba(245, 158, 11, 0.12); color: #fbbf24; border: 1px solid rgba(245, 158, 11, 0.34); }
.details-banner-info { background: rgba(56, 189, 248, 0.12); color: #38bdf8; border-color: rgba(56, 189, 248, 0.34); }
@media (max-width: 768px) {
  .inline-monitor { left: 10px; right: 10px; bottom: 10px; }
  .summary-pill { padding: 7px 10px; gap: 8px; }
  .pill-item { font-size: calc(10px + var(--ui-font-size-offset)); }
}
</style>

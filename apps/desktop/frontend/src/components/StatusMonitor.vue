<template>
  <div ref="rootEl" class="status-monitor" :class="{ 'status-monitor-empty': !activeSessionId, 'status-monitor-focus': focusPulse }">
    <div v-if="activeSessionId" class="panel-title-row">
      <h4 class="panel-title">服务器状态</h4>
      <button v-if="currentStatus" class="panel-title-action" type="button" :title="chartsEnabled ? '隐藏图表' : '显示图表'" @click="toggleCharts">
        <i class="fas fa-chart-line"></i>
      </button>
    </div>

    <div v-if="!activeSessionId" class="placeholder-state"><i class="fas fa-plug"></i><span>无活动连接</span></div>
    <div v-else-if="!statusMonitorEnabled" class="placeholder-state"><i class="fas fa-ban"></i><span>状态监控已关闭</span></div>
    <div v-else-if="isWaitingForFirstSample && !currentStatus" class="placeholder-state"><i class="fas fa-spinner fa-spin"></i><span>正在采集状态...</span></div>
    <div v-else-if="!currentStatus" class="placeholder-state error-state"><i class="fas fa-exclamation-triangle"></i><span>{{ statusError || '暂无状态数据' }}</span></div>

    <div v-else class="status-content">
      <div v-if="statusError" class="status-banner">{{ statusError }}</div>

      <div class="info-grid">
        <div v-if="showStatusMonitorIpAddress" class="status-item">
          <label class="item-label">IP</label>
          <span class="item-value item-link" :class="{ disabled: !canCopyIp }" :title="displayIpAddress" @click="copyIpToClipboard">{{ displayIpAddress }}</span>
        </div>
        <div class="status-item"><label class="item-label">CPU 型号</label><span class="item-value" :title="displayCpuModel">{{ displayCpuModel }}</span></div>
        <div class="status-item"><label class="item-label">系统</label><span class="item-value" :title="displayOsName">{{ displayOsName }}</span></div>
      </div>

      <div class="metrics-group">
        <div
          class="metric-hover-host"
          @mouseenter="handleMetricEnter('cpu', $event)"
          @mouseleave="scheduleMetricClose()"
        >
          <StatusMetricBar label="CPU" :percent="cpuPercent" :detail="cpuDetail" tone="cpu" />
        </div>

        <div
          class="metric-hover-host"
          @mouseenter="handleMetricEnter('memory', $event)"
          @mouseleave="scheduleMetricClose()"
        >
          <StatusMetricBar label="内存" :percent="memPercent" :detail="memDisplay" tone="memory" />
        </div>

        <StatusMetricBar label="Swap" :percent="swapPercent" :detail="swapDisplay" tone="swap" />

        <div
          class="metric-hover-host"
          @mouseenter="handleMetricEnter('disk', $event)"
          @mouseleave="scheduleMetricClose()"
        >
          <StatusMetricBar label="磁盘" :percent="diskPercent" :detail="diskDisplay" tone="disk" />
        </div>
      </div>

      <div class="network-row">
        <label class="item-label">网络 ({{ netInterfaceDisplay }})</label>
        <div class="network-values">
          <span class="network-rate network-down"><i class="fas fa-arrow-down"></i><span>{{ netRxDisplay }}</span></span>
          <span class="network-rate network-up"><i class="fas fa-arrow-up"></i><span>{{ netTxDisplay }}</span></span>
        </div>
      </div>

      <StatusNetworkList v-if="showInterfaceDetails && currentStatus.netInterfaces.length > 0" :interfaces="currentStatus.netInterfaces" />

      <Suspense v-if="chartsEnabled">
        <template #default>
          <StatusCharts
            :cpu-history="cpuHistory"
            :mem-used-history="memUsedHistory"
            :net-rx-history="netRxHistory"
            :net-tx-history="netTxHistory"
            :current-cpu-percent="cpuPercent"
            :current-mem-used="currentStatus.memUsed ?? 0"
            :current-net-rx-rate="currentStatus.netRxRate ?? 0"
            :current-net-tx-rate="currentStatus.netTxRate ?? 0"
          />
        </template>
        <template #fallback><div class="placeholder-state"><i class="fas fa-spinner fa-spin"></i><span>图表加载中...</span></div></template>
      </Suspense>
    </div>

    <Teleport to="body">
      <div
        v-if="hoveredDetail && hoveredPanelVisible"
        class="metric-hover-panel"
        :style="hoverPanelStyle"
        @mouseenter="cancelMetricClose()"
        @mouseleave="closeMetricPanel()"
      >
        <StatusCpuGrid
          v-if="hoveredDetail === 'cpu' && showPerCoreCpu && currentStatus?.cpuPerCore.length"
          :usages="currentStatus.cpuPerCore"
        />
        <StatusProcessList
          v-else-if="hoveredDetail === 'memory' && showTopProcesses && currentStatus?.topProcesses.length"
          :processes="currentStatus.topProcesses"
        />
        <StatusDiskList
          v-else-if="hoveredDetail === 'disk' && diskItems.length"
          :disks="diskItems"
        />
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { computed, defineAsyncComponent, onBeforeUnmount, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import StatusCpuGrid from '@/components/status/StatusCpuGrid.vue';
import StatusDiskList from '@/components/status/StatusDiskList.vue';
import StatusMetricBar from '@/components/status/StatusMetricBar.vue';
import StatusNetworkList from '@/components/status/StatusNetworkList.vue';
import StatusProcessList from '@/components/status/StatusProcessList.vue';
import { useStatusMonitor } from '@/composables/useStatusMonitor';
import { useSessionStore } from '@/stores/session';
import { useSettingsStore } from '@/stores/settings';
import { useUINotificationStore } from '@/stores/uiNotifications';

const StatusCharts = defineAsyncComponent(() => import('@/components/StatusCharts.vue'));
const sessionStore = useSessionStore();
const uiNotificationStore = useUINotificationStore();
const settingsStore = useSettingsStore();
const { activeSessionId } = storeToRefs(sessionStore);
const rootEl = ref<HTMLElement | null>(null);
const focusPulse = ref(false);
const hoveredDetail = ref<'cpu' | 'memory' | 'disk' | null>(null);
const hoveredPanelVisible = ref(false);
const hoveredAnchor = ref<{ top: number; left: number; width: number; bottom: number } | null>(null);
const cachedCpuModel = ref('');
const cachedOsName = ref('');
let focusTimer: number | null = null;
let hoverCloseTimer: number | null = null;

const {
  currentStatus,
  statusError,
  isWaitingForFirstSample,
  cpuHistory,
  memUsedHistory,
  netRxHistory,
  netTxHistory,
} = useStatusMonitor({ consumerId: 'workspace-pane' });

void settingsStore.loadAll().catch(() => undefined);

const statusMonitorEnabled = computed(() => settingsStore.getBoolean('statusMonitorEnabled', true));
const chartsEnabled = computed(() => settingsStore.getBoolean('statusMonitorShowCharts', true));
const showStatusMonitorIpAddress = computed(() => settingsStore.getBoolean('showStatusMonitorIpAddress', false));
const showTopProcesses = computed(() => settingsStore.getBoolean('statusMonitorShowTopProcesses', true));
const showPerCoreCpu = computed(() => settingsStore.getBoolean('statusMonitorShowPerCoreCpu', true));
const showInterfaceDetails = computed(() => settingsStore.getBoolean('statusMonitorShowInterfaceDetails', true));

watch(currentStatus, (value) => {
  if (value?.cpuModel?.trim()) cachedCpuModel.value = value.cpuModel.trim();
  if (value?.osName?.trim()) cachedOsName.value = value.osName.trim();
}, { immediate: true });

const displayIpAddress = computed(() => currentStatus.value?.ipAddress?.trim() || '--');
const canCopyIp = computed(() => displayIpAddress.value !== '--');
const displayCpuModel = computed(() => currentStatus.value?.cpuModel?.trim() || cachedCpuModel.value || '未知');
const displayOsName = computed(() => currentStatus.value?.osName?.trim() || cachedOsName.value || '未知');
const cpuPercent = computed(() => normalizePercent(currentStatus.value?.cpuPercent));
const memPercent = computed(() => normalizePercent(currentStatus.value?.memPercent));
const swapPercent = computed(() => normalizePercent(currentStatus.value?.swapPercent));
const diskPercent = computed(() => normalizePercent(currentStatus.value?.diskPercent));
const cpuDetail = computed(() => currentStatus.value?.cpuCores ? `${currentStatus.value.cpuCores} Cores` : '--');
const memDisplay = computed(() => formatPair(currentStatus.value?.memUsed, currentStatus.value?.memTotal, 'MB'));
const swapDisplay = computed(() => currentStatus.value?.swapTotal ? formatPair(currentStatus.value?.swapUsed, currentStatus.value?.swapTotal, 'MB') : '未启用');
const diskDisplay = computed(() => formatPair(currentStatus.value?.diskUsed, currentStatus.value?.diskTotal, 'KB'));
const diskItems = computed(() => currentStatus.value?.disks ?? []);
const netInterfaceDisplay = computed(() => currentStatus.value?.netInterface?.trim() || '...');
const netRxDisplay = computed(() => formatRate(currentStatus.value?.netRxRate));
const netTxDisplay = computed(() => formatRate(currentStatus.value?.netTxRate));
const hoverPanelStyle = computed(() => {
  const anchor = hoveredAnchor.value;
  if (!anchor || typeof window === 'undefined') {
    return undefined;
  }
  const panelWidth = Math.min(420, Math.max(300, window.innerWidth - anchor.left - 24));
  const estimatedHeight =
    hoveredDetail.value === 'cpu'
      ? 220
      : hoveredDetail.value === 'memory'
        ? 260
        : 240;
  const showAbove = anchor.bottom + 8 + estimatedHeight > window.innerHeight - 12;
  const top = showAbove ? Math.max(12, anchor.top - estimatedHeight - 8) : anchor.bottom + 8;
  return {
    top: `${top}px`,
    left: `${Math.min(anchor.left, window.innerWidth - panelWidth - 12)}px`,
    width: `${panelWidth}px`,
  } as Record<string, string>;
});

async function toggleCharts() {
  try {
    await settingsStore.set('statusMonitorShowCharts', chartsEnabled.value ? 'false' : 'true');
  } catch (error) {
    uiNotificationStore.addNotification('error', `更新图表开关失败：${error instanceof Error ? error.message : String(error)}`);
  }
}

async function copyIpToClipboard() {
  if (!canCopyIp.value) return;
  try {
    await navigator.clipboard.writeText(displayIpAddress.value);
    uiNotificationStore.addNotification('success', 'IP 地址已复制');
  } catch {
    uiNotificationStore.addNotification('error', '复制 IP 地址失败');
  }
}

function handleFocusPanel() {
  rootEl.value?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  focusPulse.value = true;
  if (focusTimer !== null) window.clearTimeout(focusTimer);
  focusTimer = window.setTimeout(() => {
    focusPulse.value = false;
    focusTimer = null;
  }, 1600);
}

onMounted(() => {
  window.addEventListener('nexus:status-monitor-focus', handleFocusPanel as EventListener);
});

onBeforeUnmount(() => {
  window.removeEventListener('nexus:status-monitor-focus', handleFocusPanel as EventListener);
  if (focusTimer !== null) window.clearTimeout(focusTimer);
  if (hoverCloseTimer !== null) window.clearTimeout(hoverCloseTimer);
});

function handleMetricEnter(kind: 'cpu' | 'memory' | 'disk', event: MouseEvent) {
  cancelMetricClose();
  const target = event.currentTarget as HTMLElement | null;
  if (!target) {
    return;
  }
  const rect = target.getBoundingClientRect();
  hoveredAnchor.value = {
    top: rect.top,
    left: rect.left + 68,
    width: rect.width,
    bottom: rect.bottom,
  };
  hoveredDetail.value = kind;
  hoveredPanelVisible.value = true;
}

function scheduleMetricClose() {
  cancelMetricClose();
  hoverCloseTimer = window.setTimeout(() => {
    closeMetricPanel();
  }, 80);
}

function cancelMetricClose() {
  if (hoverCloseTimer !== null) {
    window.clearTimeout(hoverCloseTimer);
    hoverCloseTimer = null;
  }
}

function closeMetricPanel() {
  cancelMetricClose();
  hoveredPanelVisible.value = false;
  hoveredDetail.value = null;
  hoveredAnchor.value = null;
}

function normalizePercent(value: number | undefined): number {
  if (!Number.isFinite(value)) return 0;
  return Math.round(Math.max(0, Math.min(100, Number(value))) * 10) / 10;
}

function formatPair(used: number | undefined, total: number | undefined, unit: 'MB' | 'KB'): string {
  if (!Number.isFinite(used) || !Number.isFinite(total)) return '--';
  return `${formatSize(Number(used), unit)} / ${formatSize(Number(total), unit)}`;
}

function formatSize(value: number, unit: 'MB' | 'KB'): string {
  if (unit === 'MB') return value < 1024 ? `${value.toFixed(0)} MB` : `${(value / 1024).toFixed(1)} GB`;
  return `${(value / 1024 / 1024).toFixed(1)} GB`;
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
.status-monitor { display: flex; flex-direction: column; gap: 12px; width: 100%; max-width: none; height: 100%; margin: 0; padding: 10px; overflow: auto; background: var(--bg-base, #020617); color: var(--text, #f8fafc); transition: box-shadow 0.2s ease, border-color 0.2s ease; }
.status-monitor-empty { background: var(--bg-surface0, #1e293b); }
.status-monitor-focus { box-shadow: inset 0 0 0 2px rgba(34, 197, 94, 0.9), 0 0 0 1px rgba(34, 197, 94, 0.24), 0 0 18px rgba(34, 197, 94, 0.22); }
.panel-title-row { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.panel-title { margin: 0; font-size: calc(14px + var(--ui-font-size-offset)); }
.panel-title-action { border: none; border-radius: 10px; padding: 6px 8px; background: color-mix(in srgb, var(--bg-surface0) 84%, transparent); color: var(--text); cursor: pointer; transition: background-color 0.18s ease, color 0.18s ease; }
.panel-title-action:hover { background: color-mix(in srgb, #22c55e 18%, var(--bg-surface0)); color: #22c55e; }
.placeholder-state { display: flex; align-items: center; justify-content: center; gap: 8px; min-height: 96px; border: 1px dashed var(--border); border-radius: 14px; color: var(--text-sub); background: color-mix(in srgb, var(--bg-surface0) 58%, transparent); }
.error-state { color: #f59e0b; }
.status-content { display: flex; flex-direction: column; gap: 12px; }
.status-banner { padding: 10px 12px; border: 1px solid rgba(245, 158, 11, 0.35); border-radius: 12px; background: rgba(245, 158, 11, 0.12); color: #fbbf24; }
.info-grid { display: flex; flex-direction: column; gap: 10px; }
.status-item { display: flex; flex-direction: column; gap: 4px; padding: 10px 12px; border: 1px solid var(--border); border-radius: 12px; background: color-mix(in srgb, var(--bg-surface0) 72%, transparent); min-width: 0; }
.item-label { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); }
.item-value { color: var(--text); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.item-link { cursor: pointer; }
.item-link.disabled { cursor: not-allowed; opacity: 0.6; }
.metrics-group { display: flex; flex-direction: column; gap: 10px; padding: 12px; border: 1px solid var(--border); border-radius: 14px; background: color-mix(in srgb, var(--bg-surface0) 76%, transparent); overflow: visible; }
.metric-hover-host { position: relative; }
.metric-hover-panel {
  position: fixed;
  z-index: 12020;
  padding: 10px;
  border: 1px solid color-mix(in srgb, var(--border) 82%, transparent);
  border-radius: 16px;
  background: color-mix(in srgb, var(--bg-base) 94%, transparent);
  box-shadow: 0 18px 34px color-mix(in srgb, var(--bg-base) 78%, transparent);
}
.network-row { display: flex; align-items: center; justify-content: space-between; gap: 10px; padding: 12px; border: 1px solid var(--border); border-radius: 14px; background: color-mix(in srgb, var(--bg-surface0) 76%, transparent); flex-wrap: wrap; }
.network-values { display: flex; align-items: center; gap: 12px; flex-wrap: wrap; }
.network-rate { display: inline-flex; align-items: center; gap: 6px; font-weight: 600; }
.network-down { color: #22c55e; }
.network-up { color: #38bdf8; }
@media (max-width: 860px) {
  .status-monitor { max-width: none; margin: 0; }
}
</style>

<template>
  <div class="status-monitor" :class="{ 'status-monitor-empty': !activeSessionId }">
    <h4 v-if="activeSessionId" class="panel-title">服务器状态</h4>

    <div v-if="!activeSessionId" class="placeholder-state">
      <i class="fas fa-plug"></i>
      <span>无活动连接</span>
    </div>

    <div v-else-if="statusError" class="placeholder-state error-state">
      <i class="fas fa-exclamation-triangle"></i>
      <span>{{ statusError }}</span>
    </div>

    <div v-else-if="isWaitingForFirstSample" class="placeholder-state">
      <i class="fas fa-spinner fa-spin"></i>
      <span>正在采集状态...</span>
    </div>

    <div v-else class="status-content">
      <div class="info-grid">
        <div class="status-item">
          <label class="item-label">IP:</label>
          <span
            class="item-value item-link"
            :class="{ disabled: !canCopyIp }"
            :title="displayIpAddress"
            @click="copyIpToClipboard"
          >
            {{ displayIpAddress }}
          </span>
        </div>

        <div class="status-item">
          <label class="item-label">CPU 型号:</label>
          <span class="item-value" :title="displayCpuModel">{{ displayCpuModel }}</span>
        </div>

        <div class="status-item">
          <label class="item-label">系统:</label>
          <span class="item-value" :title="displayOsName">{{ displayOsName }}</span>
        </div>
      </div>

      <div class="metrics-group">
        <div class="metric-row">
          <label class="metric-label">CPU:</label>
          <div class="metric-main">
            <div class="progress-track">
              <div
                class="progress-fill cpu"
                :class="{ 'has-value': cpuPercent > 0 }"
                :style="{ width: `${cpuPercent}%` }"
              >
                <span class="progress-text">{{ formatPercent(cpuPercent) }}</span>
              </div>
            </div>
          </div>
        </div>

        <div class="metric-row">
          <label class="metric-label">内存:</label>
          <div class="metric-main metric-main-with-detail">
            <div class="progress-track">
              <div
                class="progress-fill memory"
                :class="{ 'has-value': memPercent > 0 }"
                :style="{ width: `${memPercent}%` }"
              >
                <span class="progress-text">{{ formatPercent(memPercent) }}</span>
              </div>
            </div>
            <span class="metric-detail">{{ memDisplay }}</span>
          </div>
        </div>

        <div class="metric-row">
          <label class="metric-label">Swap:</label>
          <div class="metric-main metric-main-with-detail">
            <div class="progress-track">
              <div
                class="progress-fill"
                :class="{ 'has-value': swapPercent > 0 }"
                :style="{ width: `${swapPercent}%`, backgroundColor: swapColor }"
              >
                <span class="progress-text">{{ formatPercent(swapPercent) }}</span>
              </div>
            </div>
            <span class="metric-detail">{{ swapDisplay }}</span>
          </div>
        </div>

        <div class="metric-row">
          <label class="metric-label">磁盘:</label>
          <div class="metric-main metric-main-with-detail">
            <div class="progress-track">
              <div
                class="progress-fill disk"
                :class="{ 'has-value': diskPercent > 0 }"
                :style="{ width: `${diskPercent}%` }"
              >
                <span class="progress-text">{{ formatPercent(diskPercent) }}</span>
              </div>
            </div>
            <span class="metric-detail">{{ diskDisplay }}</span>
          </div>
        </div>
      </div>

      <div class="network-row">
        <label class="item-label">网络 ({{ netInterfaceDisplay }}):</label>
        <div class="network-values">
          <span class="network-rate network-down">
            <i class="fas fa-arrow-down"></i>
            <span>{{ netRxDisplay }}</span>
          </span>
          <span class="network-rate network-up">
            <i class="fas fa-arrow-up"></i>
            <span>{{ netTxDisplay }}</span>
          </span>
        </div>
      </div>

      <StatusCharts
        v-if="activeSessionId && currentStatus"
        :cpu-history="cpuHistory"
        :net-rx-history="netRxHistory"
        :net-tx-history="netTxHistory"
        :current-cpu-percent="cpuPercent"
        :current-net-rx-rate="currentStatus.netRxRate ?? 0"
        :current-net-tx-rate="currentStatus.netTxRate ?? 0"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';

import StatusCharts from '@/components/StatusCharts.vue';
import { useStatusMonitor } from '@/composables/useStatusMonitor';
import { useSessionStore } from '@/stores/session';
import { useUINotificationStore } from '@/stores/uiNotifications';

const sessionStore = useSessionStore();
const uiNotificationStore = useUINotificationStore();
const { activeSessionId } = storeToRefs(sessionStore);

const {
  currentStatus,
  statusError,
  isWaitingForFirstSample,
  cpuHistory,
  netRxHistory,
  netTxHistory,
} = useStatusMonitor();

const cachedCpuModel = ref<string>('');
const cachedOsName = ref<string>('');

watch(
  currentStatus,
  (value) => {
    if (!value) {
      return;
    }

    const nextCpuModel = value.cpuModel?.trim();
    const nextOsName = value.osName?.trim();
    if (nextCpuModel) {
      cachedCpuModel.value = nextCpuModel;
    }
    if (nextOsName) {
      cachedOsName.value = nextOsName;
    }
  },
  { immediate: true },
);

const displayIpAddress = computed(() => currentStatus.value?.ipAddress?.trim() || '--');
const canCopyIp = computed(() => displayIpAddress.value !== '--');
const displayCpuModel = computed(() => currentStatus.value?.cpuModel?.trim() || cachedCpuModel.value || '未知');
const displayOsName = computed(() => currentStatus.value?.osName?.trim() || cachedOsName.value || '未知');

const cpuPercent = computed(() => normalizePercent(currentStatus.value?.cpuPercent));
const memPercent = computed(() => normalizePercent(currentStatus.value?.memPercent));
const swapPercent = computed(() => normalizePercent(currentStatus.value?.swapPercent));
const diskPercent = computed(() => normalizePercent(currentStatus.value?.diskPercent));
const swapColor = computed(() => (swapPercent.value > 0 ? '#eab308' : '#6b7280'));

const memDisplay = computed(() => {
  const used = currentStatus.value?.memUsed;
  const total = currentStatus.value?.memTotal;
  if (typeof used !== 'number' || typeof total !== 'number') {
    return '--';
  }
  return `${formatMemorySize(used)} / ${formatMemorySize(total)}`;
});

const diskDisplay = computed(() => {
  const used = currentStatus.value?.diskUsed;
  const total = currentStatus.value?.diskTotal;
  if (typeof used !== 'number' || typeof total !== 'number') {
    return '--';
  }
  return `${formatDiskSize(used)} / ${formatDiskSize(total)}`;
});

const swapDisplay = computed(() => {
  const used = currentStatus.value?.swapUsed;
  const total = currentStatus.value?.swapTotal;
  if (typeof used !== 'number' || typeof total !== 'number') {
    return '--';
  }

  if (total <= 0) {
    return '未启用';
  }

  return `${formatMemorySize(used)} / ${formatMemorySize(total)}`;
});

const netInterfaceDisplay = computed(() => currentStatus.value?.netInterface?.trim() || '...');
const netRxDisplay = computed(() => formatBytesPerSecond(currentStatus.value?.netRxRate));
const netTxDisplay = computed(() => formatBytesPerSecond(currentStatus.value?.netTxRate));

function normalizePercent(value: number | undefined): number {
  if (!Number.isFinite(value)) {
    return 0;
  }
  const clamped = Math.max(0, Math.min(100, Number(value)));
  return Math.round(clamped * 10) / 10;
}

function formatPercent(value: number): string {
  return `${Math.round(value)}%`;
}

function formatMemorySize(mb: number): string {
  if (mb < 1024) {
    const rounded = Number.isInteger(mb) ? mb.toFixed(0) : mb.toFixed(1);
    return `${rounded} MB`;
  }
  return `${(mb / 1024).toFixed(1)} GB`;
}

function formatDiskSize(kb: number): string {
  return `${(kb / 1024 / 1024).toFixed(1)} GB`;
}

function formatBytesPerSecond(bytes: number | undefined): string {
  if (!Number.isFinite(bytes)) {
    return '--';
  }

  const safeBytes = Math.max(0, Number(bytes));
  if (safeBytes < 1024) {
    return `${safeBytes.toFixed(0)} B/s`;
  }
  if (safeBytes < 1024 * 1024) {
    return `${(safeBytes / 1024).toFixed(1)} KB/s`;
  }
  if (safeBytes < 1024 * 1024 * 1024) {
    return `${(safeBytes / (1024 * 1024)).toFixed(1)} MB/s`;
  }
  return `${(safeBytes / (1024 * 1024 * 1024)).toFixed(1)} GB/s`;
}

async function copyIpToClipboard() {
  if (!canCopyIp.value) {
    return;
  }

  try {
    await navigator.clipboard.writeText(displayIpAddress.value);
    uiNotificationStore.addNotification('success', 'IP 地址已复制');
  } catch {
    uiNotificationStore.addNotification('error', '复制 IP 地址失败');
  }
}
</script>

<style scoped>
.status-monitor {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-width: 0;
  height: 100%;
  padding: 12px;
  overflow-y: auto;
  overflow-x: hidden;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  font-family: 'Inter', 'Segoe UI', 'Microsoft YaHei UI', 'PingFang SC', sans-serif;
  font-size: 0.8125rem;
  line-height: 1.45;
}

.status-monitor-empty {
  background: var(--bg-surface0, #313244);
}

.panel-title {
  margin: 0 0 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid var(--border, #45475a);
  font-size: 16px;
  font-weight: 600;
  letter-spacing: 0.2px;
}

.placeholder-state {
  display: flex;
  flex: 1;
  align-items: center;
  justify-content: center;
  flex-direction: column;
  gap: 10px;
  color: var(--text-dim, #6c7086);
  text-align: center;
}

.placeholder-state i {
  font-size: 2.2em;
}

.error-state {
  color: #fda4af;
}

.status-content {
  display: flex;
  flex-direction: column;
  width: 100%;
  min-width: 0;
  gap: 10px;
  min-height: 0;
}

.info-grid {
  display: grid;
  gap: 6px;
}

.status-item {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 10px;
  min-width: 0;
}

.item-label,
.metric-label {
  color: var(--text-sub, #a6adc8);
  font-weight: 600;
  white-space: nowrap;
}

.item-value {
  min-width: 0;
  color: var(--text, #cdd6f4);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.item-link {
  color: var(--blue, #89b4fa);
  cursor: pointer;
  transition: color 0.15s ease;
}

.item-link:hover {
  color: #b4d4ff;
}

.item-link.disabled {
  color: var(--text-dim, #6c7086);
  cursor: default;
}

.metrics-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 2px;
}

.metric-row {
  display: grid;
  grid-template-columns: 42px 1fr;
  align-items: center;
  gap: 10px;
}

.metric-main {
  display: flex;
  align-items: center;
  min-width: 0;
}

.metric-main-with-detail {
  gap: 8px;
}

.progress-track {
  position: relative;
  width: 100%;
  height: 1rem;
  border-radius: 999px;
  border: 1px solid rgba(255, 255, 255, 0.04);
  background: rgba(17, 23, 34, 0.95);
  overflow: hidden;
}

.progress-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  border-radius: 999px;
  transition: width 0.28s ease;
}

.progress-fill.has-value {
  min-width: 2.4em;
}

.progress-fill.cpu {
  background: #3b82f6;
}

.progress-fill.memory {
  background: #22c55e;
}

.progress-fill.disk {
  background: #a855f7;
}

.progress-text {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #f8fafc;
  font-size: 0.6875rem;
  font-weight: 700;
  text-shadow: 0 1px 1px rgba(0, 0, 0, 0.45);
  letter-spacing: 0.1px;
  line-height: 1;
}

.metric-detail {
  flex-shrink: 1;
  min-width: 0;
  color: var(--text, #cdd6f4);
  font-size: 0.8125rem;
  font-family: 'Inter', 'Segoe UI', 'Microsoft YaHei UI', 'PingFang SC', sans-serif;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.network-row {
  display: grid;
  grid-template-columns: auto 1fr;
  align-items: center;
  gap: 10px;
  margin-top: 2px;
  min-width: 0;
}

.network-values {
  display: flex;
  align-items: center;
  gap: 14px;
  min-width: 0;
  overflow: hidden;
}

.network-rate {
  display: inline-flex;
  align-items: center;
  gap: 0.3em;
  font-size: 0.8125rem;
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace;
  white-space: nowrap;
}

.network-down {
  color: #66d08f;
}

.network-up {
  color: #ff9f59;
}

.network-rate i {
  font-size: 0.95em;
  line-height: 1;
}
</style>


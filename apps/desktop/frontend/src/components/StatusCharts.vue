<template>
  <div class="status-charts">
    <StatusLineChartCard
      title="CPU 使用率"
      :meta="`当前 ${formatPercent(currentCpuPercent)}`"
      :labels="labels"
      :datasets="cpuDatasets"
      :max-y="100"
    />
    <StatusLineChartCard
      title="内存使用"
      :meta="`当前 ${formatMemory(currentMemUsed)}`"
      :labels="labels"
      :datasets="memoryDatasets"
    />
    <StatusLineChartCard
      :title="`网络速率 (${networkUnitLabel})`"
      :meta="`↓ ${formatNetwork(currentNetRxRate)} · ↑ ${formatNetwork(currentNetTxRate)}`"
      :labels="labels"
      :datasets="networkDatasets"
      :max-y="networkChartMax"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, type PropType } from 'vue';

import StatusLineChartCard from '@/components/status/StatusLineChartCard.vue';

const MAX_DATA_POINTS = 60;

const props = defineProps({
  cpuHistory: { type: Array as PropType<ReadonlyArray<number | null>>, required: true },
  memUsedHistory: { type: Array as PropType<ReadonlyArray<number | null>>, required: true },
  netRxHistory: { type: Array as PropType<ReadonlyArray<number | null>>, required: true },
  netTxHistory: { type: Array as PropType<ReadonlyArray<number | null>>, required: true },
  currentCpuPercent: { type: Number, required: true },
  currentMemUsed: { type: Number, required: true },
  currentNetRxRate: { type: Number, required: true },
  currentNetTxRate: { type: Number, required: true },
});

const labels = Array.from({ length: MAX_DATA_POINTS }, () => '');
const safeCpuHistory = computed(() => toFixedHistory(props.cpuHistory));
const safeMemHistory = computed(() => toFixedHistory(props.memUsedHistory));
const safeNetRxHistory = computed(() => toFixedHistory(props.netRxHistory));
const safeNetTxHistory = computed(() => toFixedHistory(props.netTxHistory));

const networkUnitIsMB = computed(() => {
  const peak = Math.max(
    props.currentNetRxRate,
    props.currentNetTxRate,
    ...safeNetRxHistory.value,
    ...safeNetTxHistory.value,
  );
  return peak >= 1024 * 1024;
});
const networkUnitLabel = computed(() => (networkUnitIsMB.value ? 'MB/s' : 'KB/s'));
const networkDivisor = computed(() => (networkUnitIsMB.value ? 1024 * 1024 : 1024));
const networkPrecision = computed(() => (networkUnitIsMB.value ? 2 : 1));

const cpuDatasets = computed(() => [
  {
    label: 'CPU (%)',
    data: safeCpuHistory.value,
    borderColor: '#38bdf8',
    backgroundColor: 'rgba(56,189,248,0.18)',
  },
]);

const memoryDatasets = computed(() => [
  {
    label: '内存 (MB)',
    data: safeMemHistory.value,
    borderColor: '#22c55e',
    backgroundColor: 'rgba(34,197,94,0.18)',
  },
]);

const networkDatasets = computed(() => {
  const divisor = networkDivisor.value;
  const precision = networkPrecision.value;
  const mapValue = (value: number) => Number((value / divisor).toFixed(precision));
  return [
    {
      label: `下载 (${networkUnitLabel.value})`,
      data: safeNetRxHistory.value.map(mapValue),
      borderColor: '#22c55e',
      backgroundColor: 'rgba(34,197,94,0.18)',
    },
    {
      label: `上传 (${networkUnitLabel.value})`,
      data: safeNetTxHistory.value.map(mapValue),
      borderColor: '#38bdf8',
      backgroundColor: 'rgba(56,189,248,0.18)',
    },
  ];
});

const networkChartMax = computed(() => {
  const divisor = networkDivisor.value;
  const values = [
    ...safeNetRxHistory.value.map((value) => value / divisor),
    ...safeNetTxHistory.value.map((value) => value / divisor),
  ];
  const peak = Math.max(...values, 0);
  if (peak <= 0) {
    return networkUnitIsMB.value ? 1 : 100;
  }
  return networkUnitIsMB.value
    ? Math.max(1, Math.ceil(peak * 1.2))
    : Math.max(10, Math.ceil((peak * 1.2) / 10) * 10);
});

function toFixedHistory(history: ReadonlyArray<number | null>): number[] {
  const next = history.slice(-MAX_DATA_POINTS).map((value) => (Number.isFinite(value) ? Number(value) : 0));
  while (next.length < MAX_DATA_POINTS) {
    next.unshift(0);
  }
  return next;
}

function formatPercent(value: number): string {
  return `${Math.round(Number.isFinite(value) ? value : 0)}%`;
}

function formatMemory(value: number): string {
  if (!Number.isFinite(value) || value <= 0) return '--';
  return value < 1024 ? `${value.toFixed(0)} MB` : `${(value / 1024).toFixed(1)} GB`;
}

function formatNetwork(value: number): string {
  if (!Number.isFinite(value)) return '--';
  if (value < 1024) return `${value.toFixed(0)} B/s`;
  if (value < 1024 * 1024) return `${(value / 1024).toFixed(1)} KB/s`;
  if (value < 1024 * 1024 * 1024) return `${(value / 1024 / 1024).toFixed(1)} MB/s`;
  return `${(value / 1024 / 1024 / 1024).toFixed(1)} GB/s`;
}
</script>

<style scoped>
.status-charts { display: grid; grid-template-columns: repeat(auto-fit, minmax(260px, 1fr)); gap: 12px; }
</style>

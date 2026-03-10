<template>
  <div ref="chartsRoot" class="status-charts">
    <div class="chart-card">
      <div class="chart-head">
        <h5 class="chart-title">CPU 使用率</h5>
        <span class="chart-meta">当前 {{ latestCpuDisplay }}</span>
      </div>
      <div class="chart-wrapper">
        <Line
          :key="`cpu-${chartRenderKey}`"
          :data="cpuChartData"
          :options="cpuChartOptions"
        />
      </div>
    </div>

    <div class="chart-card">
      <div class="chart-head">
        <h5 class="chart-title">网络速率 ({{ networkUnitLabel }})</h5>
        <span class="chart-meta">↓ {{ latestNetRxDisplay }} · ↑ {{ latestNetTxDisplay }}</span>
      </div>
      <div class="chart-wrapper">
        <Line
          :key="`net-${chartRenderKey}-${networkChartMax}`"
          :data="networkChartData"
          :options="networkChartOptions"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, type PropType } from 'vue';
import { Line } from 'vue-chartjs';
import {
  CategoryScale,
  Chart as ChartJS,
  Legend,
  LineElement,
  LinearScale,
  PointElement,
  Title,
  Tooltip,
  type ChartOptions,
  type TooltipItem,
} from 'chart.js';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Title, Tooltip, Legend);

const MAX_DATA_POINTS = 60;

const props = defineProps({
  cpuHistory: {
    type: Array as PropType<ReadonlyArray<number | null>>,
    required: true,
  },
  netRxHistory: {
    type: Array as PropType<ReadonlyArray<number | null>>,
    required: true,
  },
  netTxHistory: {
    type: Array as PropType<ReadonlyArray<number | null>>,
    required: true,
  },
  currentCpuPercent: {
    type: Number,
    required: true,
  },
  currentNetRxRate: {
    type: Number,
    required: true,
  },
  currentNetTxRate: {
    type: Number,
    required: true,
  },
});

const chartsRoot = ref<HTMLElement | null>(null);
const chartRenderKey = ref(0);

let resizeObserver: ResizeObserver | null = null;
let resizeRafId = 0;
let refreshRafId = 0;
let lastObservedWidth = 0;
let lastObservedHeight = 0;

const labels = Array.from({ length: MAX_DATA_POINTS }, () => '');

const safeCpuHistory = computed(() => toFixedHistory(props.cpuHistory));
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

const cpuChartData = computed(() => ({
  labels,
  datasets: [
    {
      label: 'CPU 使用率 (%)',
      data: safeCpuHistory.value,
      borderColor: 'rgba(54, 162, 235, 1)',
      backgroundColor: 'rgba(54, 162, 235, 0.2)',
      borderWidth: 1,
      tension: 0.1,
      pointRadius: 0,
      pointHoverRadius: 5,
      fill: false,
    },
  ],
}));

const networkChartData = computed(() => {
  const divisor = networkDivisor.value;
  const precision = networkPrecision.value;
  const mapValue = (value: number) => Number((value / divisor).toFixed(precision));

  return {
    labels,
    datasets: [
      {
        label: `下载 (${networkUnitLabel.value})`,
        data: safeNetRxHistory.value.map(mapValue),
        borderColor: 'rgba(75, 192, 192, 1)',
        backgroundColor: 'rgba(75, 192, 192, 0.2)',
        borderWidth: 1,
        tension: 0.1,
        pointRadius: 0,
        pointHoverRadius: 5,
        fill: false,
      },
      {
        label: `上传 (${networkUnitLabel.value})`,
        data: safeNetTxHistory.value.map(mapValue),
        borderColor: 'rgba(255, 159, 64, 1)',
        backgroundColor: 'rgba(255, 159, 64, 0.2)',
        borderWidth: 1,
        tension: 0.1,
        pointRadius: 0,
        pointHoverRadius: 5,
        fill: false,
      },
    ],
  };
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

  let suggestedMax = peak * 1.2;

  if (networkUnitIsMB.value) {
    suggestedMax = Math.max(1, Math.ceil(suggestedMax));
  } else if (suggestedMax <= 100) {
    suggestedMax = Math.max(10, Math.ceil(suggestedMax / 10) * 10);
  } else if (suggestedMax <= 500) {
    suggestedMax = Math.ceil(suggestedMax / 50) * 50;
  } else {
    suggestedMax = Math.ceil(suggestedMax / 100) * 100;
  }

  return suggestedMax;
});

const latestCpuDisplay = computed(() => formatPercent(props.currentCpuPercent));
const latestNetRxDisplay = computed(() => formatNetworkValue(props.currentNetRxRate));
const latestNetTxDisplay = computed(() => formatNetworkValue(props.currentNetTxRate));

const baseChartOptions: Omit<ChartOptions<'line'>, 'scales'> = {
  responsive: true,
  maintainAspectRatio: false,
  animation: false,
  interaction: {
    mode: 'index',
    intersect: false,
  },
  plugins: {
    legend: {
      labels: {
        color: '#9ca3af',
      },
    },
    tooltip: {
      enabled: true,
      mode: 'index',
      intersect: false,
    },
  },
};

const cpuChartOptions = computed<ChartOptions<'line'>>(() => ({
  ...baseChartOptions,
  scales: {
    y: {
      beginAtZero: true,
      min: 0,
      max: 100,
      ticks: {
        color: '#9ca3af',
        callback: (value) => `${value}%`,
      },
      grid: {
        color: 'rgba(156, 163, 175, 0.1)',
      },
    },
    x: {
      ticks: {
        display: false,
        color: '#9ca3af',
        maxRotation: 0,
        minRotation: 0,
      },
      grid: {
        display: false,
      },
    },
  },
}));

const networkChartOptions = computed<ChartOptions<'line'>>(() => ({
  ...baseChartOptions,
  plugins: {
    ...baseChartOptions.plugins,
    tooltip: {
      ...baseChartOptions.plugins?.tooltip,
      callbacks: {
        label: (context: TooltipItem<'line'>) => {
          const datasetLabel = context.dataset.label ?? '';
          const value = context.parsed.y;
          if (value === null || value === undefined) {
            return datasetLabel;
          }
          return `${datasetLabel}: ${Number(value).toFixed(networkPrecision.value)} ${networkUnitLabel.value}`;
        },
      },
    },
  },
  scales: {
    y: {
      beginAtZero: true,
      min: 0,
      max: networkChartMax.value,
      ticks: {
        color: '#9ca3af',
        callback: (value) => {
          const precision = networkUnitIsMB.value ? 2 : 0;
          return Number(value).toFixed(precision);
        },
      },
      grid: {
        color: 'rgba(156, 163, 175, 0.1)',
      },
    },
    x: {
      ticks: {
        display: false,
        color: '#9ca3af',
        maxRotation: 0,
        minRotation: 0,
      },
      grid: {
        display: false,
      },
    },
  },
}));

function scheduleChartRefresh() {
  if (typeof window === 'undefined') {
    return;
  }

  if (refreshRafId) {
    window.cancelAnimationFrame(refreshRafId);
  }

  refreshRafId = window.requestAnimationFrame(() => {
    chartRenderKey.value += 1;
    refreshRafId = 0;
  });
}

const handleLayoutResized = () => {
  scheduleChartRefresh();
};

onMounted(() => {
  if (typeof window !== 'undefined') {
    window.addEventListener('resize', handleLayoutResized);
    window.addEventListener('nexus:layout-resized', handleLayoutResized as EventListener);
  }

  if (!chartsRoot.value || typeof ResizeObserver === 'undefined') {
    return;
  }

  resizeObserver = new ResizeObserver((entries) => {
    const entry = entries[0];
    if (!entry) {
      return;
    }

    const { width, height } = entry.contentRect;
    if (Math.abs(width - lastObservedWidth) < 1 && Math.abs(height - lastObservedHeight) < 1) {
      return;
    }

    lastObservedWidth = width;
    lastObservedHeight = height;

    if (resizeRafId) {
      cancelAnimationFrame(resizeRafId);
    }

    resizeRafId = requestAnimationFrame(() => {
      scheduleChartRefresh();
      resizeRafId = 0;
    });
  });

  resizeObserver.observe(chartsRoot.value);
});

onBeforeUnmount(() => {
  if (typeof window !== 'undefined') {
    window.removeEventListener('resize', handleLayoutResized);
    window.removeEventListener('nexus:layout-resized', handleLayoutResized as EventListener);
  }

  if (refreshRafId) {
    cancelAnimationFrame(refreshRafId);
    refreshRafId = 0;
  }

  if (resizeRafId) {
    cancelAnimationFrame(resizeRafId);
    resizeRafId = 0;
  }

  if (resizeObserver) {
    resizeObserver.disconnect();
    resizeObserver = null;
  }
});

function toFixedHistory(history: ReadonlyArray<number | null>): number[] {
  const source = history.slice(-MAX_DATA_POINTS);
  const normalized = source.map((value) => (Number.isFinite(value) ? Number(value) : 0));

  if (normalized.length >= MAX_DATA_POINTS) {
    return normalized;
  }

  return Array(MAX_DATA_POINTS - normalized.length).fill(0).concat(normalized);
}

function formatPercent(value: number): string {
  const safeValue = Number.isFinite(value) ? Math.max(0, Math.min(100, value)) : 0;
  return `${safeValue.toFixed(1)}%`;
}

function formatNetworkValue(value: number): string {
  const safeValue = Number.isFinite(value) ? Math.max(0, value) : 0;
  const converted = safeValue / networkDivisor.value;
  return `${converted.toFixed(networkPrecision.value)} ${networkUnitLabel.value}`;
}
</script>

<style scoped>
.status-charts {
  display: grid;
  grid-template-columns: 1fr;
  gap: 12px;
  margin-top: 8px;
  width: 100%;
}

.chart-card {
  min-width: 0;
  border-radius: 8px;
  border: 1px solid rgba(255, 255, 255, 0.06);
  background: #2a3140;
  padding: 8px 10px;
}

.chart-head {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
  gap: 8px;
}

.chart-title {
  margin: 0;
  color: #aeb7c9;
  font-size: calc(12px + var(--ui-font-size-offset));
  font-weight: 600;
}

.chart-meta {
  color: #8f98aa;
  font-size: calc(11px + var(--ui-font-size-offset));
  font-family: Consolas, 'Cascadia Mono', 'Microsoft YaHei UI', monospace;
  white-space: nowrap;
}

.chart-wrapper {
  position: relative;
  width: 100%;
  min-width: 0;
  height: 160px;
}

.chart-wrapper :deep(canvas) {
  width: 100% !important;
  height: 100% !important;
}
</style>

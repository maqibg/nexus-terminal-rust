<template>
  <div ref="rootEl" class="chart-card">
    <div class="chart-head">
      <h5 class="chart-title">{{ title }}</h5>
      <span class="chart-meta">{{ meta }}</span>
    </div>
    <div class="chart-wrapper">
      <Line :key="renderKey" :data="chartData" :options="chartOptions" />
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
  Tooltip,
  type ChartOptions,
} from 'chart.js';

ChartJS.register(CategoryScale, LinearScale, PointElement, LineElement, Tooltip, Legend);

interface DatasetDef {
  label: string;
  data: number[];
  borderColor: string;
  backgroundColor: string;
}

const props = defineProps({
  title: { type: String, required: true },
  meta: { type: String, required: true },
  labels: { type: Array as PropType<string[]>, required: true },
  datasets: { type: Array as PropType<DatasetDef[]>, required: true },
  maxY: { type: Number, required: false, default: undefined },
});

const rootEl = ref<HTMLElement | null>(null);
const renderKey = ref(0);
let resizeObserver: ResizeObserver | null = null;

const chartData = computed(() => ({
  labels: props.labels,
  datasets: props.datasets.map((item) => ({
    ...item,
    borderWidth: 1,
    tension: 0.14,
    pointRadius: 0,
    pointHoverRadius: 4,
    fill: false,
  })),
}));

const chartOptions = computed<ChartOptions<'line'>>(() => ({
  responsive: true,
  maintainAspectRatio: false,
  animation: false,
  interaction: { mode: 'index', intersect: false },
  plugins: {
    legend: { labels: { color: '#94a3b8' } },
    tooltip: { enabled: true, mode: 'index', intersect: false },
  },
  scales: {
    x: { ticks: { color: '#64748b', maxTicksLimit: 6 }, grid: { color: 'rgba(148,163,184,0.08)' } },
    y: {
      beginAtZero: true,
      min: 0,
      max: props.maxY,
      ticks: { color: '#94a3b8' },
      grid: { color: 'rgba(148,163,184,0.08)' },
    },
  },
}));

onMounted(() => {
  if (!rootEl.value) return;
  resizeObserver = new ResizeObserver(() => {
    renderKey.value += 1;
  });
  resizeObserver.observe(rootEl.value);
});

onBeforeUnmount(() => {
  resizeObserver?.disconnect();
  resizeObserver = null;
});
</script>

<style scoped>
.chart-card { border: 1px solid var(--border); border-radius: 14px; padding: 12px; background: color-mix(in srgb, var(--bg-surface0) 74%, transparent); min-height: 220px; }
.chart-head { display: flex; align-items: center; justify-content: space-between; gap: 10px; margin-bottom: 10px; }
.chart-title { margin: 0; font-size: calc(13px + var(--ui-font-size-offset)); }
.chart-meta { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); }
.chart-wrapper { height: 170px; }
</style>

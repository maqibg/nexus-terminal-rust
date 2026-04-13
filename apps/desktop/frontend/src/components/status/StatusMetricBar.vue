<template>
  <div class="metric-row">
    <label class="metric-label">{{ label }}</label>
    <div class="metric-main" :class="{ 'metric-main-with-detail': Boolean(detail) }">
      <div class="progress-track">
        <div
          class="progress-fill"
          :class="[toneClass, { 'has-value': percent > 0 }]"
          :style="{ width: `${percent}%` }"
        >
          <span class="progress-text">{{ Math.round(percent) }}%</span>
        </div>
      </div>
      <span v-if="detail" class="metric-detail">{{ detail }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  label: string;
  percent: number;
  detail?: string;
  tone?: 'cpu' | 'memory' | 'swap' | 'disk';
}>();

const toneClass = computed(() => `progress-fill-${props.tone ?? 'cpu'}`);
</script>

<style scoped>
.metric-row { display: flex; align-items: center; gap: 10px; }
.metric-label { width: 58px; flex: 0 0 58px; color: var(--text-sub); font-weight: 600; }
.metric-main { display: flex; align-items: center; flex: 1; min-width: 0; }
.metric-main-with-detail { gap: 10px; }
.progress-track {
  flex: 1;
  min-width: 0;
  height: 18px;
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  border-radius: 999px;
  overflow: hidden;
  background: color-mix(in srgb, var(--bg-surface0) 84%, transparent);
}
.progress-fill {
  height: 100%;
  min-width: 42px;
  display: flex;
  align-items: center;
  justify-content: flex-end;
  padding: 0 8px;
  transition: width 0.2s ease, background-color 0.2s ease;
}
.progress-fill.has-value { box-shadow: inset 0 0 0 1px color-mix(in srgb, #ffffff 10%, transparent); }
.progress-fill-cpu { background: linear-gradient(90deg, #2563eb, #0ea5e9); }
.progress-fill-memory { background: linear-gradient(90deg, #16a34a, #22c55e); }
.progress-fill-swap { background: linear-gradient(90deg, #ca8a04, #eab308); }
.progress-fill-disk { background: linear-gradient(90deg, #7c3aed, #8b5cf6); }
.progress-text { font-size: calc(11px + var(--ui-font-size-offset)); color: #f8fafc; font-weight: 700; }
.metric-detail { flex: 0 0 auto; color: var(--text); font-size: calc(12px + var(--ui-font-size-offset)); }
</style>

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
          <span class="progress-text">{{ formatPercent(percent) }}</span>
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

const toneClass = computed(() => props.tone ?? 'cpu');

function formatPercent(value: number): string {
  return `${Math.round(value)}%`;
}
</script>

<style scoped>
.metric-row {
  display: grid;
  grid-template-columns: 40px minmax(0, 1fr);
  align-items: center;
  gap: 8px;
}

.metric-label {
  color: var(--text-sub);
  font-weight: 600;
  white-space: nowrap;
}

.metric-main {
  display: flex;
  align-items: center;
  min-width: 0;
}

.metric-main-with-detail {
  display: grid;
  grid-template-columns: minmax(0, 1fr) auto;
  align-items: center;
  gap: 8px;
}

.progress-track {
  position: relative;
  width: auto;
  flex: 1 1 auto;
  min-width: 0;
  height: 1rem;
  border: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  border-radius: 999px;
  overflow: hidden;
  background: color-mix(in srgb, var(--bg-surface0) 84%, transparent);
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
  overflow: hidden;
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

.progress-fill.swap {
  background: #eab308;
}

.progress-fill.disk {
  background: #a855f7;
}

.progress-text {
  position: absolute;
  inset: 0;
  display: flex;
  align-items: center;
  justify-content: center;
  color: #f8fafc;
  font-size: calc(0.6875rem + var(--ui-font-size-offset));
  font-weight: 700;
  text-shadow: 0 1px 1px rgba(0, 0, 0, 0.45);
  letter-spacing: 0.1px;
  line-height: 1;
  transform: translateY(-1px);
  white-space: nowrap;
  pointer-events: none;
}

.metric-detail {
  flex-shrink: 0;
  min-width: max-content;
  color: var(--text);
  font-size: calc(0.6875rem + var(--ui-font-size-offset));
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, 'Liberation Mono', monospace;
  white-space: nowrap;
}

@container (max-width: 270px) {
  .metric-main-with-detail {
    grid-template-columns: minmax(0, 1fr);
    gap: 4px;
  }

  .metric-detail {
    justify-self: end;
  }
}
</style>

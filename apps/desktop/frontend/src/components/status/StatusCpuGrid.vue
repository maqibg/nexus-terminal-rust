<template>
  <section v-if="usages.length > 0" class="section-card">
    <div class="section-head">
      <h5 class="section-title">CPU 核心</h5>
      <span class="section-meta">{{ usages.length }} Cores</span>
    </div>
    <div class="core-grid">
      <div v-for="(usage, index) in usages" :key="index" class="core-item">
        <div class="core-label">Core {{ index }}</div>
        <div class="core-track">
          <div class="core-fill" :class="usageClass(usage)" :style="{ width: `${usage}%` }"></div>
        </div>
        <div class="core-value" :class="usageTextClass(usage)">{{ usage.toFixed(1) }}%</div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
defineProps<{ usages: readonly number[] }>();

function usageClass(value: number): string {
  if (value >= 90) return 'core-fill-danger';
  if (value >= 70) return 'core-fill-warn';
  return 'core-fill-ok';
}

function usageTextClass(value: number): string {
  if (value >= 90) return 'text-danger';
  if (value >= 70) return 'text-warn';
  return 'text-ok';
}
</script>

<style scoped>
.section-card { border: 1px solid var(--border); border-radius: 14px; padding: 12px; background: color-mix(in srgb, var(--bg-surface0) 74%, transparent); }
.section-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 10px; }
.section-title { margin: 0; font-size: calc(13px + var(--ui-font-size-offset)); }
.section-meta { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); }
.core-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(110px, 1fr)); gap: 10px; }
.core-item { border: 1px solid color-mix(in srgb, var(--border) 72%, transparent); border-radius: 10px; padding: 8px; background: color-mix(in srgb, var(--bg-base) 70%, transparent); }
.core-label { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); margin-bottom: 6px; }
.core-track { height: 8px; border-radius: 999px; background: color-mix(in srgb, var(--bg-mantle) 78%, transparent); overflow: hidden; }
.core-fill { height: 100%; transition: width 0.2s ease; }
.core-fill-ok { background: linear-gradient(90deg, #16a34a, #22c55e); }
.core-fill-warn { background: linear-gradient(90deg, #d97706, #f59e0b); }
.core-fill-danger { background: linear-gradient(90deg, #dc2626, #ef4444); }
.core-value { margin-top: 6px; font-size: calc(12px + var(--ui-font-size-offset)); font-weight: 700; }
.text-ok { color: #22c55e; }
.text-warn { color: #f59e0b; }
.text-danger { color: #ef4444; }
</style>

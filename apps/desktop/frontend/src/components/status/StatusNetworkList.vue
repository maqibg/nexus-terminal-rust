<template>
  <section v-if="interfaces.length > 0" class="section-card">
    <div class="section-head">
      <h5 class="section-title">网络接口</h5>
      <span class="section-meta">{{ interfaces.length }} Interfaces</span>
    </div>
    <div class="list">
      <div v-for="item in interfaces" :key="item.name" class="row">
        <span class="row-label">{{ item.name }}</span>
        <div class="row-values">
          <span class="down">↓ {{ formatRate(item.rxRate) }}</span>
          <span class="up">↑ {{ formatRate(item.txRate) }}</span>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { NetInterfaceEntry } from '@/lib/api-status';

defineProps<{ interfaces: ReadonlyArray<NetInterfaceEntry> }>();

function formatRate(bytes: number): string {
  if (bytes < 1024) return `${bytes.toFixed(0)} B/s`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB/s`;
  if (bytes < 1024 * 1024 * 1024) return `${(bytes / 1024 / 1024).toFixed(1)} MB/s`;
  return `${(bytes / 1024 / 1024 / 1024).toFixed(1)} GB/s`;
}
</script>

<style scoped>
.section-card { border: 1px solid var(--border); border-radius: 14px; padding: 12px; background: color-mix(in srgb, var(--bg-surface0) 74%, transparent); }
.section-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 10px; }
.section-title { margin: 0; font-size: calc(13px + var(--ui-font-size-offset)); }
.section-meta { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); }
.list { display: flex; flex-direction: column; gap: 8px; }
.row { display: flex; align-items: center; justify-content: space-between; gap: 12px; padding: 8px 10px; border-radius: 10px; background: color-mix(in srgb, var(--bg-base) 68%, transparent); }
.row-label { color: var(--text); font-weight: 600; }
.row-values { display: flex; align-items: center; gap: 10px; font-size: calc(11px + var(--ui-font-size-offset)); }
.down { color: #22c55e; }
.up { color: #38bdf8; }
</style>

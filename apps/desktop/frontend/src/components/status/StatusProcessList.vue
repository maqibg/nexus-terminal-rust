<template>
  <section v-if="processes.length > 0" class="section-card">
    <div class="section-head">
      <h5 class="section-title">内存占用前十进程</h5>
      <span class="section-meta">Top {{ processes.length }}</span>
    </div>
    <div class="list">
      <div v-for="item in processes" :key="`${item.pid}-${item.command}`" class="row">
        <div class="row-main">
          <span class="row-command" :title="item.command">{{ item.command }}</span>
          <span class="row-percent">{{ item.memPercent.toFixed(1) }}%</span>
        </div>
        <div class="row-sub">PID {{ item.pid }}</div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { StatusProcessEntry } from '@/lib/api-status';

defineProps<{ processes: ReadonlyArray<StatusProcessEntry> }>();
</script>

<style scoped>
.section-card { border: 1px solid var(--border); border-radius: 14px; padding: 12px; background: color-mix(in srgb, var(--bg-surface0) 74%, transparent); }
.section-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 10px; }
.section-title { margin: 0; font-size: calc(13px + var(--ui-font-size-offset)); }
.section-meta { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); }
.list { display: flex; flex-direction: column; gap: 8px; max-height: 220px; overflow-y: auto; }
.row { padding: 8px 10px; border-radius: 10px; background: color-mix(in srgb, var(--bg-base) 68%, transparent); }
.row-main { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.row-command { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; color: var(--text); font-family: 'Fira Code', Consolas, monospace; }
.row-percent { color: #22c55e; font-weight: 700; white-space: nowrap; }
.row-sub { margin-top: 4px; color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); }
</style>

<template>
  <section v-if="disks.length > 0" class="section-card">
    <div class="section-head">
      <h5 class="section-title">磁盘详情</h5>
      <span class="section-meta">{{ disks.length }} Volumes</span>
    </div>
    <div class="list">
      <div v-for="disk in disks" :key="disk.name" class="row">
        <div class="row-main">
          <span class="row-label" :title="disk.name">{{ disk.name }}</span>
          <span class="row-meta">{{ formatSize(disk.usedKb) }} / {{ formatSize(disk.totalKb) }}</span>
        </div>
        <div class="row-track">
          <div class="row-fill" :style="{ width: `${clamp(disk.percent)}%` }"></div>
        </div>
      </div>
    </div>
  </section>
</template>

<script setup lang="ts">
import type { DiskUsageEntry } from '@/lib/api-status';

defineProps<{ disks: ReadonlyArray<DiskUsageEntry> }>();

function clamp(value: number): number {
  return Math.max(0, Math.min(100, Number.isFinite(value) ? value : 0));
}

function formatSize(kb: number): string {
  return `${(kb / 1024 / 1024).toFixed(1)} GB`;
}
</script>

<style scoped>
.section-card { border: 1px solid var(--border); border-radius: 14px; padding: 12px; background: color-mix(in srgb, var(--bg-surface0) 74%, transparent); }
.section-head { display: flex; align-items: center; justify-content: space-between; gap: 8px; margin-bottom: 10px; }
.section-title { margin: 0; font-size: calc(13px + var(--ui-font-size-offset)); }
.section-meta { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); }
.list { display: flex; flex-direction: column; gap: 8px; }
.row-main { display: flex; align-items: center; justify-content: space-between; gap: 10px; }
.row-label { color: var(--text); font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.row-meta { color: var(--text-sub); font-size: calc(11px + var(--ui-font-size-offset)); white-space: nowrap; }
.row-track { margin-top: 6px; height: 8px; border-radius: 999px; background: color-mix(in srgb, var(--bg-mantle) 78%, transparent); overflow: hidden; }
.row-fill { height: 100%; background: linear-gradient(90deg, #7c3aed, #8b5cf6); }
</style>

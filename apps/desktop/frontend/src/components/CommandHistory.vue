<template>
  <div class="history-panel">
    <div class="history-header">
      <span>命令历史</span>
      <button class="btn-icon" @click="handleClear" title="清空" v-if="items.length">×</button>
    </div>
    <div v-if="loading" class="status">加载中...</div>
    <div v-else class="history-list">
      <div v-for="item in items" :key="item.id" class="history-item" @click="$emit('execute', item.command)">
        <span class="history-cmd">{{ item.command }}</span>
        <span class="history-time">{{ formatTime(item.timestamp) }}</span>
      </div>
      <div v-if="!items.length" class="status">暂无历史</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { historyApi } from '@/lib/api';
import type { CommandHistory } from '@/lib/api';

defineEmits<{ execute: [command: string] }>();

const items = ref<CommandHistory[]>([]);
const loading = ref(false);

async function load() {
  loading.value = true;
  try { items.value = await historyApi.list(100, 0); } catch { /* ignore */ }
  finally { loading.value = false; }
}

async function handleClear() {
  if (!confirm('确定清空命令历史？')) return;
  try { await historyApi.clear(); items.value = []; } catch { /* ignore */ }
}

function formatTime(t: string): string {
  try { return new Date(t).toLocaleTimeString(); } catch { return t; }
}

onMounted(load);
</script>

<style scoped>
.history-panel { display: flex; flex-direction: column; height: 100%; }
.history-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 6px 10px; border-bottom: 1px solid var(--border); font-size: calc(0.8rem + var(--ui-font-size-offset)); color: var(--text-sub);
}
.btn-icon {
  width: 20px; height: 20px; border-radius: 4px; border: 1px solid var(--border);
  background: transparent; color: var(--red); cursor: pointer; font-size: calc(0.8rem + var(--ui-font-size-offset));
  display: flex; align-items: center; justify-content: center;
}
.btn-icon:hover { background: var(--bg-surface1); }
.history-list { flex: 1; overflow-y: auto; padding: 4px; }
.history-item {
  padding: 4px 8px; border-radius: 4px; cursor: pointer;
  display: flex; justify-content: space-between; align-items: center; gap: 8px; margin-bottom: 1px;
}
.history-item:hover { background: var(--bg-surface1); }
.history-cmd { font-size: calc(0.75rem + var(--ui-font-size-offset)); color: var(--blue); font-family: monospace; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.history-time { font-size: calc(0.65rem + var(--ui-font-size-offset)); color: var(--text-dim); flex-shrink: 0; }
.status { padding: 8px; text-align: center; color: var(--text-dim); font-size: calc(0.75rem + var(--ui-font-size-offset)); }
</style>

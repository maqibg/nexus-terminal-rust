<template>
  <div class="page">
    <div class="page-header">
      <h2>命令历史</h2>
      <div class="actions">
        <input v-model="search" class="input" placeholder="搜索..." />
        <button class="btn btn-danger" @click="handleClear">清空</button>
      </div>
    </div>
    <div v-for="h in filtered" :key="h.id" class="history-item">
      <span class="cmd">{{ h.command }}</span>
      <span class="time">{{ new Date(h.timestamp).toLocaleString() }}</span>
    </div>
    <div v-if="!filtered.length" class="empty">暂无记录</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useCommandHistoryStore } from '@/stores/commandHistory';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { storeToRefs } from 'pinia';

const store = useCommandHistoryStore();
const { items } = storeToRefs(store);
const { confirm } = useConfirmDialog();
const search = ref('');

const filtered = computed(() => {
  const q = search.value.toLowerCase();
  if (!q) return items.value;
  return items.value.filter(h => h.command.toLowerCase().includes(q));
});

async function handleClear() {
  if (await confirm('清空历史', '确定清空所有命令历史吗？')) await store.clear();
}

onMounted(() => store.fetchAll(200));
</script>

<style scoped>
.page { padding: 20px; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.page-header h2 { font-size: calc(18px + var(--ui-font-size-offset)); color: var(--text); }
.actions { display: flex; gap: 8px; }
.input { padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: calc(13px + var(--ui-font-size-offset)); }
.history-item { display: flex; justify-content: space-between; padding: 8px 12px; border-bottom: 1px solid var(--border); }
.cmd { font-family: monospace; font-size: calc(13px + var(--ui-font-size-offset)); color: var(--text); }
.time { font-size: calc(11px + var(--ui-font-size-offset)); color: var(--text-dim); white-space: nowrap; }
.btn-danger { padding: 6px 14px; background: var(--bg-surface1); color: var(--red); border: none; border-radius: 4px; cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); }
.empty { text-align: center; color: var(--text-dim); padding: 40px; font-size: calc(14px + var(--ui-font-size-offset)); }
</style>

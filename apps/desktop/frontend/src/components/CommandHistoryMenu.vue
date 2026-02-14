<template>
  <div class="history-menu" v-if="visible">
    <div class="menu-header">
      <input v-model="search" class="input" placeholder="搜索命令..." />
    </div>
    <div class="menu-list">
      <div v-for="h in filtered" :key="h.id" class="menu-item" @click="emit('select', h.command)">
        {{ h.command }}
      </div>
      <div v-if="!filtered.length" class="empty">无匹配</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useCommandHistoryStore } from '@/stores/commandHistory';
import { storeToRefs } from 'pinia';

defineProps<{ visible: boolean }>();
const emit = defineEmits<{ select: [cmd: string] }>();

const store = useCommandHistoryStore();
const { items } = storeToRefs(store);
const search = ref('');

const filtered = computed(() => {
  const q = search.value.toLowerCase();
  const list = q ? items.value.filter(h => h.command.toLowerCase().includes(q)) : items.value;
  return list.slice(0, 50);
});

onMounted(() => { if (!items.value.length) store.fetchAll(100); });
</script>

<style scoped>
.history-menu { position: absolute; bottom: 100%; left: 0; width: 320px; max-height: 240px; background: var(--bg-surface0); border: 1px solid var(--border); border-radius: 6px; box-shadow: 0 -4px 12px rgba(0,0,0,0.3); display: flex; flex-direction: column; z-index: 100; }
.menu-header { padding: 6px; border-bottom: 1px solid var(--border); }
.input { width: 100%; padding: 4px 8px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 3px; color: var(--text); font-size: 12px; }
.menu-list { flex: 1; overflow-y: auto; }
.menu-item { padding: 6px 10px; font-size: 12px; font-family: monospace; color: var(--text); cursor: pointer; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.menu-item:hover { background: var(--bg-surface1); }
.empty { padding: 12px; text-align: center; color: var(--text-dim); font-size: 12px; }
</style>

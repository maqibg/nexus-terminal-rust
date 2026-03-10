<template>
  <div class="connection-list">
    <div class="search-bar">
      <input v-model="search" class="input" placeholder="搜索连接..." />
    </div>
    <div class="list-body">
      <div v-for="conn in filtered" :key="conn.id" class="conn-item" :class="{ active: conn.id === activeId }" @dblclick="emit('connect', conn)" @contextmenu.prevent="onContext($event, conn)">
        <div class="conn-name">{{ conn.name }}</div>
        <div class="conn-host">{{ conn.host }}:{{ conn.port }}</div>
      </div>
      <div v-if="!filtered.length" class="empty">无匹配连接</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useConnectionsStore } from '@/stores/connections';
import { storeToRefs } from 'pinia';
import type { Connection } from '@/lib/api';

const emit = defineEmits<{ connect: [conn: Connection]; edit: [conn: Connection]; delete: [conn: Connection] }>();

const store = useConnectionsStore();
const { list } = storeToRefs(store);
const search = ref('');
const activeId = ref<number | null>(null);

const filtered = computed(() => {
  const q = search.value.toLowerCase();
  if (!q) return list.value;
  return list.value.filter(c => c.name.toLowerCase().includes(q) || c.host.toLowerCase().includes(q));
});

function onContext(e: MouseEvent, conn: Connection) {
  activeId.value = conn.id;
  // Context menu handled by parent or future FileManagerContextMenu pattern
}

onMounted(() => store.fetch());
</script>

<style scoped>
.connection-list { display: flex; flex-direction: column; height: 100%; }
.search-bar { padding: 8px; border-bottom: 1px solid var(--border); }
.input { width: 100%; padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: calc(13px + var(--ui-font-size-offset)); }
.list-body { flex: 1; overflow-y: auto; }
.conn-item { padding: 8px 12px; cursor: pointer; border-bottom: 1px solid var(--border); }
.conn-item:hover { background: var(--bg-surface0); }
.conn-item.active { background: var(--bg-surface1); }
.conn-name { font-size: calc(13px + var(--ui-font-size-offset)); color: var(--text); }
.conn-host { font-size: calc(11px + var(--ui-font-size-offset)); color: var(--text-dim); }
.empty { padding: 16px; text-align: center; color: var(--text-dim); font-size: calc(13px + var(--ui-font-size-offset)); }
</style>

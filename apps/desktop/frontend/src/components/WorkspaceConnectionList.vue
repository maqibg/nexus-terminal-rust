<template>
  <div class="ws-conn-list">
    <div v-for="conn in list" :key="conn.id" class="ws-conn-item" @click="emit('select', conn)">
      <span class="ws-conn-name">{{ conn.name }}</span>
      <span class="ws-conn-host">{{ (conn.type || 'SSH') }} · {{ conn.host }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import { useConnectionsStore } from '@/stores/connections';
import { storeToRefs } from 'pinia';
import type { Connection } from '@/lib/api';

const emit = defineEmits<{ select: [conn: Connection] }>();
const store = useConnectionsStore();
const { list } = storeToRefs(store);
onMounted(() => store.fetch());
</script>

<style scoped>
.ws-conn-list { padding: 4px 0; }
.ws-conn-item { display: flex; justify-content: space-between; padding: 6px 12px; cursor: pointer; font-size: 13px; }
.ws-conn-item:hover { background: var(--bg-surface0); }
.ws-conn-name { color: var(--text); }
.ws-conn-host { color: var(--text-dim); font-size: 11px; }
</style>


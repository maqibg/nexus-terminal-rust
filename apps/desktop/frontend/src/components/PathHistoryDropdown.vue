<template>
  <div class="path-history" v-if="items.length">
    <button class="btn-icon" @click="open = !open" title="路径历史">⏱</button>
    <div v-if="open" class="dropdown">
      <div v-for="p in items" :key="p.id" class="dropdown-item" @click="select(p.path)">{{ p.path }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { pathHistoryApi, type PathHistory } from '@/lib/api';

const props = defineProps<{ connectionId?: number }>();
const emit = defineEmits<{ navigate: [path: string] }>();

const items = ref<PathHistory[]>([]);
const open = ref(false);

onMounted(async () => {
  try { items.value = await pathHistoryApi.list(50, props.connectionId); } catch { /* ignore */ }
});

function select(path: string) { open.value = false; emit('navigate', path); }
</script>

<style scoped>
.path-history { position: relative; }
.btn-icon { width: 26px; height: 26px; border-radius: 4px; border: 1px solid var(--border); background: transparent; color: var(--text); cursor: pointer; font-size: calc(0.8rem + var(--ui-font-size-offset)); display: flex; align-items: center; justify-content: center; }
.btn-icon:hover { background: var(--bg-surface1); }
.dropdown { position: absolute; top: 100%; right: 0; background: var(--bg-surface0); border: 1px solid var(--border); border-radius: 4px; max-height: 200px; overflow-y: auto; z-index: 10; min-width: 200px; margin-top: 2px; }
.dropdown-item { padding: 4px 10px; font-size: calc(12px + var(--ui-font-size-offset)); cursor: pointer; color: var(--text); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
.dropdown-item:hover { background: var(--bg-surface1); }
</style>

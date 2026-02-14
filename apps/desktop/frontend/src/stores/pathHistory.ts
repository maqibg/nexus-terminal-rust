import { defineStore } from 'pinia';
import { ref } from 'vue';
import { pathHistoryApi, type PathHistory } from '@/lib/api';

export const usePathHistoryStore = defineStore('pathHistory', () => {
  const items = ref<PathHistory[]>([]);
  const loading = ref(false);

  async function fetchAll(connectionId?: number, limit?: number) {
    loading.value = true;
    try { items.value = await pathHistoryApi.list(connectionId, limit); }
    finally { loading.value = false; }
  }

  async function add(path: string, connectionId?: number) {
    await pathHistoryApi.add(path, connectionId);
  }

  async function clear() {
    await pathHistoryApi.clear();
    items.value = [];
  }

  return { items, loading, fetchAll, add, clear };
});

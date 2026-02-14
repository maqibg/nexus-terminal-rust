import { defineStore } from 'pinia';
import { ref } from 'vue';
import { historyApi, type CommandHistory } from '@/lib/api';

export const useCommandHistoryStore = defineStore('commandHistory', () => {
  const items = ref<CommandHistory[]>([]);
  const loading = ref(false);

  async function fetchAll(limit?: number, offset?: number) {
    loading.value = true;
    try { items.value = await historyApi.list(limit, offset); }
    finally { loading.value = false; }
  }

  async function add(command: string, sessionId?: string, connectionId?: number) {
    await historyApi.add(command, sessionId, connectionId);
  }

  async function clear() {
    await historyApi.clear();
    items.value = [];
  }

  return { items, loading, fetchAll, add, clear };
});

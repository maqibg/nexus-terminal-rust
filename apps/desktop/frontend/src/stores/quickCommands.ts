import { defineStore } from 'pinia';
import { ref } from 'vue';
import { quickCommandApi, type QuickCommand } from '@/lib/api';

export const useQuickCommandsStore = defineStore('quickCommands', () => {
  const items = ref<QuickCommand[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try { items.value = await quickCommandApi.list(); }
    finally { loading.value = false; }
  }

  async function create(data: { name: string; command: string; variables?: string; tags?: string[] }) {
    await quickCommandApi.create(data);
    await fetchAll();
  }

  async function update(id: number, data: { name: string; command: string; variables?: string; tags?: string[] }) {
    await quickCommandApi.update(id, data);
    await fetchAll();
  }

  async function remove(id: number) {
    await quickCommandApi.delete(id);
    items.value = items.value.filter(q => q.id !== id);
  }

  async function use(id: number) {
    await quickCommandApi.use(id);
  }

  return { items, loading, fetchAll, create, update, remove, use };
});

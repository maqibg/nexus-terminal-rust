import { defineStore } from 'pinia';
import { ref } from 'vue';
import { quickCommandTagApi, type QuickCommandTag } from '@/lib/api';

export const useQuickCommandTagsStore = defineStore('quickCommandTags', () => {
  const items = ref<QuickCommandTag[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try { items.value = await quickCommandTagApi.list(); }
    finally { loading.value = false; }
  }

  async function create(name: string) {
    await quickCommandTagApi.create(name);
    await fetchAll();
  }

  async function remove(id: number) {
    await quickCommandTagApi.delete(id);
    items.value = items.value.filter(t => t.id !== id);
  }

  async function bulkAssign(tagId: number, quickCommandIds: number[]) {
    await quickCommandTagApi.bulkAssign(tagId, quickCommandIds);
  }

  return { items, loading, fetchAll, create, remove, bulkAssign };
});

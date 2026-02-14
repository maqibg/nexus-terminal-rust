import { defineStore } from 'pinia';
import { ref } from 'vue';
import { connectionsApi, type Tag } from '@/lib/api';

export const useTagsStore = defineStore('tags', () => {
  const items = ref<Tag[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try { items.value = await connectionsApi.tagList(); }
    finally { loading.value = false; }
  }

  async function create(name: string) {
    await connectionsApi.tagCreate(name);
    await fetchAll();
  }

  async function remove(id: number) {
    await connectionsApi.tagDelete(id);
    items.value = items.value.filter(t => t.id !== id);
  }

  return { items, loading, fetchAll, create, remove };
});

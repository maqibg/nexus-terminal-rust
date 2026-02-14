import { defineStore } from 'pinia';
import { ref } from 'vue';
import { favoritePathApi, type FavoritePath } from '@/lib/api';

export const useFavoritePathsStore = defineStore('favoritePaths', () => {
  const items = ref<FavoritePath[]>([]);
  const loading = ref(false);

  async function fetchAll(connectionId?: number) {
    loading.value = true;
    try { items.value = await favoritePathApi.list(connectionId); }
    finally { loading.value = false; }
  }

  async function create(name: string, path: string, connectionId?: number) {
    await favoritePathApi.create(name, path, connectionId);
    await fetchAll(connectionId);
  }

  async function update(id: number, name: string, path: string, connectionId?: number) {
    await favoritePathApi.update(id, name, path, connectionId);
    await fetchAll(connectionId);
  }

  async function remove(id: number) {
    await favoritePathApi.delete(id);
    items.value = items.value.filter(f => f.id !== id);
  }

  return { items, loading, fetchAll, create, update, remove };
});

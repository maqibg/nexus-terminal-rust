import { defineStore } from 'pinia';
import { ref } from 'vue';
import { favoritePathApi, type FavoritePath } from '@/lib/api';

export const useFavoritePathsStore = defineStore('favoritePaths', () => {
  const items = ref<FavoritePath[]>([]);
  const loading = ref(false);

  async function fetchAll(connectionId?: number) {
    loading.value = true;
    try {
      items.value = await favoritePathApi.list(connectionId);
    } finally {
      loading.value = false;
    }
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
    items.value = items.value.filter((f) => f.id !== id);
  }

  async function markUsed(id: number) {
    const ok = await favoritePathApi.markUsed(id);
    if (ok) {
      const now = new Date().toISOString();
      items.value = items.value.map((f) => (f.id === id ? { ...f, last_used_at: now } : f));
    }
    return ok;
  }

  return { items, loading, fetchAll, create, update, remove, markUsed };
});

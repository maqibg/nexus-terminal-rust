import { defineStore } from 'pinia';
import { ref } from 'vue';
import { quickCommandTagApi, type QuickCommandTag } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

export const useQuickCommandTagsStore = defineStore('quickCommandTags', () => {
  const CACHE_KEY = 'quickCommandTagsCache';
  const items = ref<QuickCommandTag[]>([]);
  const loading = ref(false);
  const notificationStore = useUINotificationStore();

  function loadFromCache(): void {
    try {
      const raw = localStorage.getItem(CACHE_KEY);
      if (!raw) {
        return;
      }
      const parsed = JSON.parse(raw) as unknown;
      if (!Array.isArray(parsed)) {
        return;
      }
      items.value = parsed as QuickCommandTag[];
    } catch {
      localStorage.removeItem(CACHE_KEY);
    }
  }

  async function fetchAll() {
    if (!items.value.length) {
      loadFromCache();
    }

    loading.value = true;
    try {
      const fresh = await quickCommandTagApi.list();
      const freshJson = JSON.stringify(fresh);
      const currentJson = JSON.stringify(items.value);
      if (freshJson !== currentJson) {
        items.value = fresh;
        localStorage.setItem(CACHE_KEY, freshJson);
      }
    } catch {
      notificationStore.addNotification('error', '加载快捷指令标签失败');
    }
    finally { loading.value = false; }
  }

  async function create(name: string) {
    await quickCommandTagApi.create(name);
    localStorage.removeItem(CACHE_KEY);
    await fetchAll();
  }

  async function remove(id: number) {
    await quickCommandTagApi.delete(id);
    items.value = items.value.filter(t => t.id !== id);
    localStorage.setItem(CACHE_KEY, JSON.stringify(items.value));
  }

  async function bulkAssign(tagId: number, quickCommandIds: number[]) {
    await quickCommandTagApi.bulkAssign(tagId, quickCommandIds);
  }

  return { items, loading, fetchAll, create, remove, bulkAssign };
});

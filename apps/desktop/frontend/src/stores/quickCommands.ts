import { defineStore } from 'pinia';
import { ref } from 'vue';
import { quickCommandApi, type QuickCommand } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

export const useQuickCommandsStore = defineStore('quickCommands', () => {
  const CACHE_KEY = 'quickCommandsListCache';
  const items = ref<QuickCommand[]>([]);
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
      items.value = parsed as QuickCommand[];
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
      const fresh = await quickCommandApi.list();
      const freshJson = JSON.stringify(fresh);
      const currentJson = JSON.stringify(items.value);
      if (freshJson !== currentJson) {
        items.value = fresh;
        localStorage.setItem(CACHE_KEY, freshJson);
      }
    } catch {
      notificationStore.addNotification('error', '加载快捷指令失败');
    }
    finally { loading.value = false; }
  }

  async function create(data: { name: string; command: string; variables?: string; tags?: string[] }) {
    await quickCommandApi.create(data);
    localStorage.removeItem(CACHE_KEY);
    await fetchAll();
  }

  async function update(id: number, data: { name: string; command: string; variables?: string; tags?: string[] }) {
    await quickCommandApi.update(id, data);
    localStorage.removeItem(CACHE_KEY);
    await fetchAll();
  }

  async function remove(id: number) {
    await quickCommandApi.delete(id);
    items.value = items.value.filter(q => q.id !== id);
    localStorage.setItem(CACHE_KEY, JSON.stringify(items.value));
  }

  async function use(id: number) {
    await quickCommandApi.use(id);
  }

  return { items, loading, fetchAll, create, update, remove, use };
});

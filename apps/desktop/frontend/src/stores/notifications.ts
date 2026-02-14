import { defineStore } from 'pinia';
import { ref } from 'vue';
import { settingsApi, type NotificationChannel } from '@/lib/api';

export const useNotificationsStore = defineStore('notifications', () => {
  const items = ref<NotificationChannel[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try { items.value = await settingsApi.notificationChannelList(); }
    finally { loading.value = false; }
  }

  async function create(data: Record<string, unknown>) {
    await settingsApi.notificationChannelCreate(data);
    await fetchAll();
  }

  async function update(data: Record<string, unknown>) {
    await settingsApi.notificationChannelUpdate(data);
    await fetchAll();
  }

  async function remove(id: number) {
    await settingsApi.notificationChannelDelete(id);
    items.value = items.value.filter(n => n.id !== id);
  }

  return { items, loading, fetchAll, create, update, remove };
});

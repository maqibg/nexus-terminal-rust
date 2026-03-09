import { defineStore } from 'pinia';
import { ref } from 'vue';
import { settingsApi, type NotificationChannel } from '@/lib/api';
import { useUiNotificationsStore } from './uiNotifications';
import { toAppError } from '@/lib/errors';

export const useNotificationsStore = defineStore('notifications', () => {
  const items = ref<NotificationChannel[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try {
      items.value = await settingsApi.notificationChannelList();
    } catch (e: unknown) {
      const ui = useUiNotificationsStore();
      ui.addNotification('error', toAppError(e).message);
    } finally {
      loading.value = false;
    }
  }

  async function create(data: Record<string, unknown>) {
    try {
      await settingsApi.notificationChannelCreate(data);
      await fetchAll();
    } catch (e: unknown) {
      const ui = useUiNotificationsStore();
      ui.addNotification('error', toAppError(e).message);
    }
  }

  async function update(data: Record<string, unknown>) {
    try {
      await settingsApi.notificationChannelUpdate(data);
      await fetchAll();
    } catch (e: unknown) {
      const ui = useUiNotificationsStore();
      ui.addNotification('error', toAppError(e).message);
    }
  }

  async function remove(id: number) {
    try {
      await settingsApi.notificationChannelDelete(id);
      items.value = items.value.filter(n => n.id !== id);
    } catch (e: unknown) {
      const ui = useUiNotificationsStore();
      ui.addNotification('error', toAppError(e).message);
    }
  }

  return { items, loading, fetchAll, create, update, remove };
});

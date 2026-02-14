import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface UINotification {
  id: number;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
}

let nextId = 0;

export const useUINotificationStore = defineStore('uiNotifications', () => {
  const notifications = ref<UINotification[]>([]);

  function addNotification(type: UINotification['type'], message: string, duration = 3000) {
    const id = ++nextId;
    notifications.value.push({ id, type, message });
    if (duration > 0) {
      setTimeout(() => removeNotification(id), duration);
    }
  }

  function removeNotification(id: number) {
    notifications.value = notifications.value.filter(n => n.id !== id);
  }

  return { notifications, addNotification, removeNotification };
});

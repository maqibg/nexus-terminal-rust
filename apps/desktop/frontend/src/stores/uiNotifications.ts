import { defineStore } from 'pinia';
import { ref } from 'vue';

export interface UINotification {
  id: number;
  type: 'success' | 'error' | 'warning' | 'info';
  message: string;
}

let nextId = 0;

type NotificationInput =
  | UINotification['type']
  | {
      type: UINotification['type'];
      message: string;
      duration?: number;
    };

export const useUiNotificationsStore = defineStore('uiNotifications', () => {
  const notifications = ref<UINotification[]>([]);

  function addNotification(input: NotificationInput, message?: string, duration = 3000) {
    let finalType: UINotification['type'];
    let finalMessage: string;
    let finalDuration = duration;

    if (typeof input === 'string') {
      finalType = input;
      finalMessage = message ?? '';
    } else {
      finalType = input.type;
      finalMessage = input.message;
      finalDuration = input.duration ?? duration;
    }

    if (!finalMessage.trim()) {
      return;
    }

    const id = ++nextId;
    notifications.value.push({ id, type: finalType, message: finalMessage });
    if (finalDuration > 0) {
      setTimeout(() => removeNotification(id), finalDuration);
    }
  }

  function removeNotification(id: number) {
    notifications.value = notifications.value.filter(n => n.id !== id);
  }

  return { notifications, addNotification, removeNotification };
});

export const useUINotificationStore = useUiNotificationsStore;

import { useUINotificationStore } from '@/stores/uiNotifications';

export function useCopyText() {
  const notifications = useUINotificationStore();

  return async (text: string) => {
    const trimmed = (text ?? '').trim();
    if (!trimmed) {
      notifications.addNotification('warning', '没有可复制的内容');
      return;
    }

    try {
      await navigator.clipboard.writeText(trimmed);
      notifications.addNotification('success', '已复制到剪贴板');
    } catch {
      notifications.addNotification('error', '复制失败');
    }
  };
}

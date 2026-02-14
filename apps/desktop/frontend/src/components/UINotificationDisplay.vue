<template>
  <div class="notification-container">
    <div
      v-for="n in notifications"
      :key="n.id"
      class="notification-item"
      :class="n.type"
      @click="remove(n.id)"
    >
      <span class="notification-icon">{{ icons[n.type] }}</span>
      <span class="notification-message">{{ n.message }}</span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useUINotificationStore } from '@/stores/uiNotifications';

const store = useUINotificationStore();
const { notifications } = storeToRefs(store);
const remove = store.removeNotification;

const icons: Record<string, string> = {
  success: '✓',
  error: '✕',
  warning: '⚠',
  info: 'ℹ',
};
</script>

<style scoped>
.notification-container {
  position: fixed;
  top: 16px;
  right: 16px;
  z-index: 9999;
  display: flex;
  flex-direction: column;
  gap: 8px;
  max-width: 360px;
}
.notification-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  border-radius: 6px;
  background: var(--bg-surface0);
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
  animation: slideIn 0.2s ease;
  border-left: 3px solid transparent;
}
.notification-item.success { border-left-color: var(--green); }
.notification-item.error { border-left-color: var(--red); }
.notification-item.warning { border-left-color: var(--yellow); }
.notification-item.info { border-left-color: var(--blue); }
.notification-icon { font-size: 14px; flex-shrink: 0; }
.notification-item.success .notification-icon { color: var(--green); }
.notification-item.error .notification-icon { color: var(--red); }
.notification-item.warning .notification-icon { color: var(--yellow); }
.notification-item.info .notification-icon { color: var(--blue); }

@keyframes slideIn {
  from { transform: translateX(100%); opacity: 0; }
  to { transform: translateX(0); opacity: 1; }
}
</style>

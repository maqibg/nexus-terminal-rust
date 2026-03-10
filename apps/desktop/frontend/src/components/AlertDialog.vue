<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="close">
      <div class="dialog-card">
        <div class="dialog-title">{{ title }}</div>
        <div class="dialog-message">{{ message }}</div>
        <div class="dialog-actions">
          <button class="btn btn-primary" @click="close">{{ confirmText }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
withDefaults(defineProps<{
  visible: boolean;
  title?: string;
  message: string;
  confirmText?: string;
}>(), {
  title: '提示',
  confirmText: '确定',
});
const emit = defineEmits<{ close: [] }>();
function close() { emit('close'); }
</script>

<style scoped>
.dialog-backdrop {
  position: fixed; inset: 0; z-index: 9000;
  background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center;
}
.dialog-card {
  background: var(--bg-surface0); border-radius: 8px; padding: 24px;
  min-width: 320px; max-width: 480px; border: 1px solid var(--border);
}
.dialog-title { font-size: calc(16px + var(--ui-font-size-offset)); font-weight: 600; margin-bottom: 12px; color: var(--text); }
.dialog-message { font-size: calc(14px + var(--ui-font-size-offset)); color: var(--text-sub); margin-bottom: 20px; line-height: 1.5; }
.dialog-actions { display: flex; justify-content: flex-end; }
.btn { min-width: 72px; padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); }
.btn-primary { background: var(--blue); color: var(--bg-base); }
.btn-primary:hover { opacity: 0.9; }
</style>

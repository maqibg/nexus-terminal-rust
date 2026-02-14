<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="cancel">
      <div class="dialog-card">
        <div class="dialog-title">{{ title }}</div>
        <div class="dialog-message">{{ message }}</div>
        <div class="dialog-actions">
          <button class="btn btn-cancel" @click="cancel">{{ cancelText }}</button>
          <button class="btn btn-confirm" @click="confirm">{{ confirmText }}</button>
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
  cancelText?: string;
}>(), { title: '确认', confirmText: '确认', cancelText: '取消' });

const emit = defineEmits<{ confirm: []; cancel: [] }>();
function confirm() { emit('confirm'); }
function cancel() { emit('cancel'); }
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
.dialog-title { font-size: 16px; font-weight: 600; margin-bottom: 12px; color: var(--text); }
.dialog-message { font-size: 14px; color: var(--text-sub); margin-bottom: 20px; line-height: 1.5; }
.dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-cancel:hover { background: var(--bg-surface0); }
.btn-confirm { background: var(--blue); color: var(--bg-base); }
.btn-confirm:hover { opacity: 0.9; }
</style>

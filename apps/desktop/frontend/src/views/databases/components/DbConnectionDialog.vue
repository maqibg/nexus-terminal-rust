<template>
  <Teleport to="body">
    <div v-if="visible" class="db-dialog-backdrop" @click.self="emit('close')">
      <div class="db-dialog-panel db-dialog-scope">
        <h3 class="db-dialog-title">{{ title }}</h3>

        <div v-if="error" class="error">{{ error }}</div>

        <div class="db-dialog-body">
          <slot />
        </div>

        <div class="db-dialog-actions">
          <button type="button" class="btn btn-secondary" :disabled="busy" @click="emit('close')">取消</button>
          <button type="button" class="btn btn-primary" :disabled="busy" @click="emit('save')">
            {{ saveLabel }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
const props = withDefaults(
  defineProps<{
    visible: boolean;
    title: string;
    saveLabel?: string;
    busy?: boolean;
    error?: string;
  }>(),
  {
    saveLabel: '保存',
    busy: false,
    error: '',
  },
);

const emit = defineEmits<{
  close: [];
  save: [];
}>();
</script>

<style scoped>
.db-dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--ui-overlay);
}

.db-dialog-panel {
  width: min(100%, 720px);
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 18px;
  border: 1px solid var(--ui-dialog-border);
  border-radius: 10px;
  background: var(--ui-dialog-bg);
  box-shadow: 0 24px 64px var(--ui-dialog-shadow);
  overflow: auto;
}

.db-dialog-title {
  margin: 0;
  text-align: center;
  font-size: calc(18px + var(--ui-font-size-offset));
  font-weight: 650;
  color: var(--ui-text-strong);
}

.db-dialog-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.db-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>

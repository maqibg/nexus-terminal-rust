<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import AppSelect from './AppSelect.vue';
import SshKeyManagementModal from './SshKeyManagementModal.vue';
import { useSshKeysStore } from '@/stores/sshKeys';

const props = defineProps<{ modelValue: number | null }>();
const emit = defineEmits<{ (e: 'update:modelValue', value: number | null): void }>();

const sshKeysStore = useSshKeysStore();
const { items, loading } = storeToRefs(sshKeysStore);

const selectedKeyId = ref<number | null>(props.modelValue);
const isManagementModalVisible = ref(false);

const keys = computed(() => items.value);
const keyOptions = computed(() => [
  { value: null, label: '请选择 SSH 密钥' },
  ...keys.value.map(key => ({ value: key.id, label: key.name })),
]);

watch(() => props.modelValue, (newValue) => {
  selectedKeyId.value = newValue;
});

watch(selectedKeyId, (newValue) => {
  emit('update:modelValue', newValue ?? null);
});

watch(keys, (newKeys) => {
  if (selectedKeyId.value === null) {
    return;
  }
  const exists = newKeys.some(key => key.id === selectedKeyId.value);
  if (!exists) {
    selectedKeyId.value = null;
  }
});

const openManagementModal = async () => {
  isManagementModalVisible.value = true;
  await sshKeysStore.fetchAll();
};

const closeManagementModal = async () => {
  isManagementModalVisible.value = false;
  await sshKeysStore.fetchAll();
};

onMounted(async () => {
  if (keys.value.length === 0) {
    await sshKeysStore.fetchAll();
  }
});
</script>

<template>
  <div class="ssh-key-selector">
    <div class="selector-row">
      <AppSelect
        id="ssh-key-select"
        v-model="selectedKeyId"
        class="selector-select"
        :options="keyOptions"
        :disabled="loading"
        aria-label="SSH 密钥"
      />

      <button
        type="button"
        class="manage-btn"
        :disabled="loading"
        title="管理 SSH 密钥"
        @click="openManagementModal"
      >
        <i class="fas fa-cog"></i>
      </button>
    </div>

    <div v-if="loading" class="status-text">SSH 密钥加载中...</div>

    <SshKeyManagementModal
      :visible="isManagementModalVisible"
      @close="closeManagementModal"
    />
  </div>
</template>

<style scoped>
.ssh-key-selector {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.selector-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.selector-select {
  flex: 1;
  min-width: 0;
}

.selector-select :deep(.app-select-trigger) {
  padding: 8px 12px;
  min-height: 0;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: 13px;
}

.selector-select :deep(.app-select-trigger:focus-visible) {
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.manage-btn {
  width: 34px;
  height: 34px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text-sub);
  cursor: pointer;
}

.manage-btn:hover {
  background: var(--bg-surface1);
  color: var(--text);
}

.manage-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.status-text {
  font-size: 12px;
  color: var(--text-sub);
}
</style>

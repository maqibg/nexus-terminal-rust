<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import SshKeyManagementModal from './SshKeyManagementModal.vue';
import { useSshKeysStore } from '@/stores/sshKeys';

const props = defineProps<{ modelValue: number | null }>();
const emit = defineEmits<{ (e: 'update:modelValue', value: number | null): void }>();

const sshKeysStore = useSshKeysStore();
const { items, loading } = storeToRefs(sshKeysStore);

const selectedKeyId = ref<number | null>(props.modelValue);
const isManagementModalVisible = ref(false);

const keys = computed(() => items.value);

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
      <select
        id="ssh-key-select"
        v-model="selectedKeyId"
        class="selector-input"
        :disabled="loading"
      >
        <option :value="null">请选择 SSH 密钥</option>
        <option v-for="key in keys" :key="key.id" :value="key.id">
          {{ key.name }}
        </option>
      </select>

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

.selector-input {
  flex: 1;
  min-width: 0;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: 13px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3e%3cpath fill='none' stroke='%238e98a0' stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M2 5l6 6 6-6'/%3e%3c/svg%3e");
  background-repeat: no-repeat;
  background-size: 14px 10px;
  background-position: right 10px center;
  padding-right: 30px;
}

.selector-input:focus {
  outline: none;
  border-color: var(--blue);
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

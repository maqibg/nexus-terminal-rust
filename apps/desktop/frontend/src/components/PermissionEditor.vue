<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop">
      <div class="permission-card">
        <h3 class="permission-title">修改权限 · {{ targetName }}</h3>
        <p class="permission-hint">勾选要授予的权限，八进制值会同步更新。</p>

        <div v-for="group in GROUPS" :key="group.key" class="permission-row">
          <span class="permission-group">{{ group.label }}</span>
          <label v-for="bit in group.bits" :key="bit.key" class="permission-bit">
            <input
              type="checkbox"
              :checked="(mode & bit.value) !== 0"
              @change="toggleBit(bit.value)"
            />
            <span>{{ bit.label }}</span>
          </label>
        </div>

        <div class="permission-octal">
          <span>八进制</span>
          <strong>{{ octalText }}</strong>
        </div>

        <div class="permission-actions">
          <button class="btn btn-cancel" type="button" @click="emit('cancel')">取消</button>
          <button class="btn btn-save" type="button" @click="emit('save', mode)">确定</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  visible: boolean;
  targetName: string;
  mode: number;
}>();

const emit = defineEmits<{
  (e: 'cancel'): void;
  (e: 'save', mode: number): void;
  (e: 'update:mode', mode: number): void;
}>();

const GROUPS = [
  {
    key: 'owner',
    label: '所有者',
    bits: [
      { key: 'r', label: '读', value: 0o400 },
      { key: 'w', label: '写', value: 0o200 },
      { key: 'x', label: '执行', value: 0o100 },
    ],
  },
  {
    key: 'group',
    label: '群组',
    bits: [
      { key: 'r', label: '读', value: 0o040 },
      { key: 'w', label: '写', value: 0o020 },
      { key: 'x', label: '执行', value: 0o010 },
    ],
  },
  {
    key: 'other',
    label: '其他',
    bits: [
      { key: 'r', label: '读', value: 0o004 },
      { key: 'w', label: '写', value: 0o002 },
      { key: 'x', label: '执行', value: 0o001 },
    ],
  },
];

const octalText = computed(() => (props.mode & 0o7777).toString(8).padStart(4, '0'));

function toggleBit(bit: number) {
  const next = (props.mode & bit) !== 0 ? props.mode & ~bit : props.mode | bit;
  emit('update:mode', next & 0o7777);
}
</script>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9000;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
}

.permission-card {
  width: min(420px, 92vw);
  background: var(--bg-surface0, #1e1e2e);
  border: 1px solid var(--border, #313244);
  border-radius: 10px;
  padding: 18px 20px;
  color: var(--text, #cdd6f4);
}

.permission-title {
  margin: 0 0 6px;
  font-size: 15px;
}

.permission-hint {
  margin: 0 0 14px;
  font-size: 12px;
  color: var(--text-sub, #a6adc8);
}

.permission-row {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 6px 0;
}

.permission-group {
  width: 52px;
  font-size: 13px;
  color: var(--text-sub, #a6adc8);
}

.permission-bit {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  cursor: pointer;
}

.permission-octal {
  margin-top: 12px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 10px;
  border-radius: 6px;
  background: rgba(137, 180, 250, 0.12);
  font-size: 13px;
}

.permission-octal strong {
  font-family: monospace;
  font-size: 15px;
}

.permission-actions {
  margin-top: 16px;
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.btn {
  padding: 6px 14px;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  font-size: 13px;
}

.btn-cancel {
  background: var(--bg-surface1, #313244);
  color: var(--text-sub, #a6adc8);
}

.btn-save {
  background: var(--blue, #89b4fa);
  color: #11111b;
}
</style>

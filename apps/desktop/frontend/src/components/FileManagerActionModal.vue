<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="emit('close')">
      <div class="dialog-card">
        <div class="dialog-title">{{ title }}</div>
        <div v-if="mode !== 'delete'" class="field">
          <input ref="inputRef" v-model="inputValue" class="input" :placeholder="placeholder" @keydown.enter="submit" />
        </div>
        <div v-else class="dialog-message">确定删除 <strong>{{ targetName }}</strong> 吗？</div>
        <div class="dialog-actions">
          <button class="btn btn-cancel" @click="emit('close')">取消</button>
          <button class="btn btn-confirm" @click="submit">确认</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue';

const props = defineProps<{
  visible: boolean;
  mode: 'create_dir' | 'rename' | 'delete';
  targetName?: string;
}>();
const emit = defineEmits<{ close: []; confirm: [value: string] }>();

const inputRef = ref<HTMLInputElement>();
const inputValue = ref('');

const title = computed(() => {
  if (props.mode === 'create_dir') return '新建文件夹';
  if (props.mode === 'rename') return '重命名';
  return '删除确认';
});
const placeholder = computed(() => props.mode === 'create_dir' ? '文件夹名称' : '新名称');

watch(() => props.visible, async (v) => {
  if (v) {
    inputValue.value = props.mode === 'rename' ? (props.targetName ?? '') : '';
    await nextTick();
    inputRef.value?.focus();
  }
});

function submit() { emit('confirm', inputValue.value); }
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.dialog-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 320px; border: 1px solid var(--border); }
.dialog-title { font-size: 16px; font-weight: 600; margin-bottom: 12px; color: var(--text); }
.dialog-message { font-size: 14px; color: var(--text-sub); margin-bottom: 16px; }
.field { margin-bottom: 16px; }
.input { width: 100%; padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
.dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-confirm { background: var(--blue); color: var(--bg-base); }
</style>

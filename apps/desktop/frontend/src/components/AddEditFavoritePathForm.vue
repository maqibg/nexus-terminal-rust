<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="emit('close')">
      <div class="dialog-card">
        <div class="dialog-title">{{ editData ? '编辑收藏路径' : '新建收藏路径' }}</div>
        <label class="field"><span class="label">名称</span><input v-model="name" class="input" /></label>
        <label class="field"><span class="label">路径</span><input v-model="path" class="input" placeholder="/home/user" /></label>
        <div class="dialog-actions">
          <button class="btn btn-cancel" @click="emit('close')">取消</button>
          <button class="btn btn-confirm" @click="handleSave">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import type { FavoritePath } from '@/lib/api';

const props = defineProps<{ visible: boolean; editData?: FavoritePath | null; connectionId?: number }>();
const emit = defineEmits<{ close: []; save: [data: { name: string; path: string }] }>();

const name = ref('');
const path = ref('');

watch(() => props.visible, (v) => {
  if (v && props.editData) { name.value = props.editData.name; path.value = props.editData.path; }
  else if (v) { name.value = ''; path.value = ''; }
});

function handleSave() {
  if (!name.value.trim() || !path.value.trim()) return;
  emit('save', { name: name.value, path: path.value });
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.dialog-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 360px; border: 1px solid var(--border); }
.dialog-title { font-size: 16px; font-weight: 600; margin-bottom: 16px; color: var(--text); }
.field { display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px; }
.label { font-size: 12px; color: var(--text-sub); }
.input { padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
.dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-confirm { background: var(--blue); color: var(--bg-base); }
</style>

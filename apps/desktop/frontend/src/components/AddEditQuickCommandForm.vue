<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="emit('close')">
      <div class="dialog-card">
        <div class="dialog-title">{{ editData ? '编辑命令' : '新建命令' }}</div>
        <label class="field"><span class="label">名称</span><input v-model="name" class="input" /></label>
        <label class="field"><span class="label">命令</span><textarea v-model="command" class="input textarea" rows="3" /></label>
        <label class="field"><span class="label">变量 (JSON)</span><textarea v-model="variables" class="input textarea" rows="2" placeholder='{"var1":"默认值"}' /></label>
        <label class="field">
          <span class="label">标签</span>
          <TagInput v-model="selectedTags" :available-tags="availableTags.map(t => t.name)" />
        </label>
        <div class="dialog-actions">
          <button class="btn btn-cancel" @click="emit('close')">取消</button>
          <button class="btn btn-confirm" @click="handleSave">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch, onMounted } from 'vue';
import { storeToRefs } from 'pinia';
import { useQuickCommandTagsStore } from '@/stores/quickCommandTags';
import TagInput from './TagInput.vue';
import type { QuickCommand } from '@/lib/api';

const props = defineProps<{ visible: boolean; editData?: QuickCommand | null }>();
const emit = defineEmits<{ close: []; save: [data: { name: string; command: string; variables?: string; tags?: string[] }] }>();

const name = ref('');
const command = ref('');
const variables = ref('');
const selectedTags = ref<string[]>([]);

const tagsStore = useQuickCommandTagsStore();
const { items: availableTags } = storeToRefs(tagsStore);

onMounted(() => tagsStore.fetchAll());

watch(() => props.visible, (v) => {
  if (v && props.editData) {
    name.value = props.editData.name;
    command.value = props.editData.command;
    variables.value = props.editData.variables ?? '';
    selectedTags.value = [...(props.editData.tags ?? [])];
  } else if (v) {
    name.value = ''; command.value = ''; variables.value = ''; selectedTags.value = [];
  }
});

function handleSave() {
  if (!name.value.trim() || !command.value.trim()) return;
  emit('save', { name: name.value, command: command.value, variables: variables.value || undefined, tags: selectedTags.value.length ? selectedTags.value : undefined });
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.dialog-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 440px; border: 1px solid var(--border); }
.dialog-title { font-size: 16px; font-weight: 600; margin-bottom: 16px; color: var(--text); }
.field { display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px; }
.label { font-size: 12px; color: var(--text-sub); }
.input { padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
.textarea { resize: vertical; font-family: monospace; }
.dialog-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-confirm { background: var(--blue); color: var(--bg-base); }
</style>

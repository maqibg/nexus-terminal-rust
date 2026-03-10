<template>
  <div class="page">
    <div class="page-header">
      <h2>标签管理</h2>
      <div class="add-row">
        <input v-model="newTag" class="input" placeholder="新标签名称" @keydown.enter="handleCreate" />
        <button class="btn btn-primary" @click="handleCreate" :disabled="!newTag.trim()">添加</button>
      </div>
    </div>
    <div v-for="t in items" :key="t.id" class="tag-item">
      <span class="tag-name">{{ t.name }}</span>
      <button class="btn-sm btn-danger" @click="handleDelete(t)">删除</button>
    </div>
    <div v-if="!items.length" class="empty">暂无标签</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useTagsStore } from '@/stores/tags';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { storeToRefs } from 'pinia';

const store = useTagsStore();
const { items } = storeToRefs(store);
const { confirm } = useConfirmDialog();
const newTag = ref('');

async function handleCreate() {
  if (!newTag.value.trim()) return;
  await store.create(newTag.value.trim());
  newTag.value = '';
}
async function handleDelete(t: { id: number; name: string }) {
  if (await confirm('删除标签', `确定删除 "${t.name}" 吗？`)) await store.remove(t.id);
}
onMounted(() => store.fetchAll());
</script>

<style scoped>
.page { padding: 20px; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.page-header h2 { font-size: calc(18px + var(--ui-font-size-offset)); color: var(--text); }
.add-row { display: flex; gap: 8px; }
.input { padding: 6px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: calc(13px + var(--ui-font-size-offset)); }
.tag-item { display: flex; justify-content: space-between; align-items: center; padding: 8px 12px; border-bottom: 1px solid var(--border); }
.tag-name { font-size: calc(13px + var(--ui-font-size-offset)); color: var(--text); }
.btn-primary { padding: 6px 14px; background: var(--blue); color: var(--bg-base); border: none; border-radius: 4px; cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); }
.btn-primary:disabled { opacity: 0.5; }
.btn-sm { padding: 3px 8px; background: var(--bg-surface1); color: var(--text-sub); border: none; border-radius: 3px; cursor: pointer; font-size: calc(12px + var(--ui-font-size-offset)); }
.btn-danger { color: var(--red); }
.empty { text-align: center; color: var(--text-dim); padding: 40px; font-size: calc(14px + var(--ui-font-size-offset)); }
</style>

<template>
  <div class="page">
    <div class="page-header">
      <h2>快捷命令</h2>
      <button class="btn btn-primary" @click="showCreate = true">新建</button>
    </div>
    <table class="table" v-if="items.length">
      <thead><tr><th>名称</th><th>命令</th><th>使用次数</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="q in items" :key="q.id">
          <td>{{ q.name }}</td>
          <td class="cmd-cell">{{ q.command }}</td>
          <td>{{ q.usage_count }}</td>
          <td class="row-actions">
            <button class="btn-sm" @click="editItem = q; showCreate = true">编辑</button>
            <button class="btn-sm btn-danger" @click="handleDelete(q)">删除</button>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-else class="empty">暂无快捷命令</div>

    <!-- Inline create/edit dialog -->
    <Teleport to="body">
      <div v-if="showCreate" class="dialog-backdrop" @click.self="closeForm">
        <div class="dialog-card">
          <div class="dialog-title">{{ editItem ? '编辑命令' : '新建命令' }}</div>
          <label class="field"><span class="label">名称</span><input v-model="formName" class="input" /></label>
          <label class="field"><span class="label">命令</span><textarea v-model="formCmd" class="input textarea" rows="3" /></label>
          <div class="dialog-actions">
            <button class="btn btn-cancel" @click="closeForm">取消</button>
            <button class="btn btn-confirm" @click="handleSave">保存</button>
          </div>
        </div>
      </div>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useQuickCommandsStore } from '@/stores/quickCommands';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { storeToRefs } from 'pinia';
import type { QuickCommand } from '@/lib/api';

const store = useQuickCommandsStore();
const { items } = storeToRefs(store);
const { confirm } = useConfirmDialog();

const showCreate = ref(false);
const editItem = ref<QuickCommand | null>(null);
const formName = ref('');
const formCmd = ref('');

function closeForm() { showCreate.value = false; editItem.value = null; formName.value = ''; formCmd.value = ''; }

async function handleSave() {
  if (!formName.value.trim() || !formCmd.value.trim()) return;
  if (editItem.value) {
    await store.update(editItem.value.id, { name: formName.value, command: formCmd.value });
  } else {
    await store.create({ name: formName.value, command: formCmd.value });
  }
  closeForm();
}

async function handleDelete(q: QuickCommand) {
  if (await confirm('删除命令', `确定删除 "${q.name}" 吗？`)) await store.remove(q.id);
}

onMounted(() => store.fetchAll());
</script>

<style scoped>
.page { padding: 20px; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.page-header h2 { font-size: 18px; color: var(--text); }
.table { width: 100%; border-collapse: collapse; }
.table th, .table td { padding: 8px 12px; text-align: left; border-bottom: 1px solid var(--border); font-size: 13px; }
.table th { color: var(--text-sub); font-weight: 500; }
.cmd-cell { font-family: monospace; max-width: 300px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.row-actions { display: flex; gap: 4px; }
.btn-primary { padding: 6px 14px; background: var(--blue); color: var(--bg-base); border: none; border-radius: 4px; cursor: pointer; font-size: 13px; }
.btn-sm { padding: 3px 8px; background: var(--bg-surface1); color: var(--text-sub); border: none; border-radius: 3px; cursor: pointer; font-size: 12px; }
.btn-danger { color: var(--red); }
.empty { text-align: center; color: var(--text-dim); padding: 40px; font-size: 14px; }
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.dialog-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 400px; border: 1px solid var(--border); }
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

<template>
  <div class="page">
    <div class="page-header">
      <h2>挂起会话</h2>
    </div>
    <table class="table" v-if="items.length">
      <thead><tr><th>连接名称</th><th>挂起时间</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="s in items" :key="s.id">
          <td>{{ s.connection_name }}</td>
          <td>{{ new Date(s.suspended_at).toLocaleString() }}</td>
          <td class="row-actions">
            <button class="btn-sm btn-resume" @click="handleResume(s.id)">恢复</button>
            <button class="btn-sm btn-danger" @click="handleTerminate(s.id)">终止</button>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-else class="empty">暂无挂起会话</div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { sshSuspendApi, type SuspendedSession } from '@/lib/api-ssh-suspend';
import { useConfirmDialog } from '@/composables/useConfirmDialog';

const items = ref<SuspendedSession[]>([]);
const { confirm } = useConfirmDialog();

async function load() { items.value = await sshSuspendApi.list(); }

async function handleResume(id: string) {
  await sshSuspendApi.resume(id);
  await load();
}

async function handleTerminate(id: string) {
  if (await confirm('终止会话', '确定终止此挂起会话吗？')) {
    await sshSuspendApi.terminate(id);
    await load();
  }
}

onMounted(load);
</script>

<style scoped>
.page { padding: 20px; }
.page-header { display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px; }
.page-header h2 { font-size: 18px; color: var(--text); }
.table { width: 100%; border-collapse: collapse; }
.table th, .table td { padding: 8px 12px; text-align: left; border-bottom: 1px solid var(--border); font-size: 13px; }
.table th { color: var(--text-sub); font-weight: 500; }
.row-actions { display: flex; gap: 4px; }
.btn-sm { padding: 3px 8px; background: var(--bg-surface1); color: var(--text-sub); border: none; border-radius: 3px; cursor: pointer; font-size: 12px; }
.btn-resume { color: var(--green); }
.btn-danger { color: var(--red); }
.empty { text-align: center; color: var(--text-dim); padding: 40px; font-size: 14px; }
</style>

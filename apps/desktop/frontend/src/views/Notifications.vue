<template>
  <div class="page">
    <div class="page-header">
      <h2>通知渠道</h2>
      <button class="btn btn-primary" @click="showForm = true">新建</button>
    </div>
    <table class="table" v-if="items.length">
      <thead><tr><th>名称</th><th>类型</th><th>启用</th><th>操作</th></tr></thead>
      <tbody>
        <tr v-for="n in items" :key="n.id">
          <td>{{ n.name }}</td>
          <td>{{ n.channel_type }}</td>
          <td><span :class="n.enabled ? 'on' : 'off'">{{ n.enabled ? '是' : '否' }}</span></td>
          <td class="row-actions">
            <button class="btn-sm btn-danger" @click="handleDelete(n)">删除</button>
          </td>
        </tr>
      </tbody>
    </table>
    <div v-else class="empty">暂无通知渠道</div>
    <NotificationSettingForm :visible="showForm" @cancel="showForm = false" @saved="onSaved" />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useNotificationsStore } from '@/stores/notifications';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { storeToRefs } from 'pinia';
import NotificationSettingForm from '@/components/NotificationSettingForm.vue';

const store = useNotificationsStore();
const { items } = storeToRefs(store);
const { confirm } = useConfirmDialog();
const showForm = ref(false);

async function onSaved() { showForm.value = false; await store.fetchAll(); }
async function handleDelete(n: { id: number; name: string }) {
  if (await confirm('删除渠道', `确定删除 "${n.name}" 吗？`)) await store.remove(n.id);
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
.row-actions { display: flex; gap: 4px; }
.on { color: var(--green); }
.off { color: var(--text-dim); }
.btn-primary { padding: 6px 14px; background: var(--blue); color: var(--bg-base); border: none; border-radius: 4px; cursor: pointer; font-size: 13px; }
.btn-sm { padding: 3px 8px; background: var(--bg-surface1); color: var(--text-sub); border: none; border-radius: 3px; cursor: pointer; font-size: 12px; }
.btn-danger { color: var(--red); }
.empty { text-align: center; color: var(--text-dim); padding: 40px; font-size: 14px; }
</style>

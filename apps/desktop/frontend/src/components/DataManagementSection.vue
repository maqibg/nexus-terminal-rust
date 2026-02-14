<template>
  <section class="data-section">
    <h3 class="section-title">数据管理</h3>

    <div class="actions">
      <button class="btn" @click="exportData">导出连接</button>
      <button class="btn" @click="triggerImport">导入连接</button>
      <input type="file" ref="fileInput" hidden accept=".json" @change="importData" />
    </div>

    <div v-if="status" class="status" :class="statusType">{{ status }}</div>
  </section>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { connectionsApi } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const notify = useUINotificationStore();
const fileInput = ref<HTMLInputElement>();
const status = ref('');
const statusType = ref('');

async function exportData() {
  try {
    const json = await connectionsApi.export();
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url; a.download = 'connections-export.json'; a.click();
    URL.revokeObjectURL(url);
    notify.addNotification('success', '导出成功');
  } catch (e: any) { notify.addNotification('error', `导出失败: ${e.message}`); }
}

function triggerImport() { fileInput.value?.click(); }

async function importData(e: Event) {
  const file = (e.target as HTMLInputElement).files?.[0];
  if (!file) return;
  try {
    const text = await file.text();
    const ids = await connectionsApi.import(text);
    status.value = `成功导入 ${ids.length} 个连接`;
    statusType.value = 'success';
    notify.addNotification('success', status.value);
  } catch (e: any) {
    status.value = `导入失败: ${e.message}`;
    statusType.value = 'error';
    notify.addNotification('error', status.value);
  }
  if (fileInput.value) fileInput.value.value = '';
}
</script>

<style scoped>
.data-section { display: flex; flex-direction: column; gap: 12px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.actions { display: flex; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: 1px solid var(--border); background: var(--bg-surface0); color: var(--text); cursor: pointer; font-size: 13px; }
.btn:hover { background: var(--bg-surface1); }
.status { font-size: 13px; padding: 8px; border-radius: 4px; }
.status.success { color: var(--green); background: rgba(166,227,161,0.1); }
.status.error { color: var(--red); background: rgba(243,139,168,0.1); }
</style>

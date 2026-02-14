<template>
  <section class="focus-section">
    <h3 class="section-title">焦点切换快捷键</h3>
    <div v-for="(shortcut, panel) in shortcuts" :key="panel" class="shortcut-row">
      <label>{{ panelLabel(panel) }}</label>
      <input class="input" v-model="shortcuts[panel]" placeholder="例如 Ctrl+1" @blur="save" />
    </div>
  </section>
</template>

<script setup lang="ts">
import { reactive, onMounted } from 'vue';
import { settingsApi } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const notify = useUINotificationStore();
const shortcuts = reactive<Record<string, string>>({ terminal: '', sftp: '', editor: '' });

onMounted(async () => {
  try {
    const all = await settingsApi.getAll();
    const stored = all.find(s => s.key === 'focus_shortcuts');
    if (stored) Object.assign(shortcuts, JSON.parse(stored.value));
  } catch { /* ignore */ }
});

function panelLabel(key: string): string {
  const map: Record<string, string> = { terminal: '终端', sftp: '文件管理', editor: '编辑器' };
  return map[key] ?? key;
}

async function save() {
  try {
    await settingsApi.set('focus_shortcuts', JSON.stringify(shortcuts));
    notify.addNotification('success', '快捷键已保存');
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.focus-section { display: flex; flex-direction: column; gap: 10px; }
.section-title { font-size: 15px; font-weight: 600; margin: 0; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.shortcut-row { display: flex; align-items: center; gap: 12px; }
.shortcut-row label { font-size: 13px; color: var(--text-sub); min-width: 80px; }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 6px 8px; color: var(--text); font-size: 13px; outline: none; width: 140px; }
.input:focus { border-color: var(--blue); }
</style>

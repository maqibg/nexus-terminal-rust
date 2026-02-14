<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('close')">
      <div class="modal-card">
        <div class="modal-header">
          <span>挂起会话管理</span>
          <span class="close-btn" @click="$emit('close')">&times;</span>
        </div>
        <div class="session-list">
          <div v-for="s in sessions" :key="s.id" class="session-item">
            <div class="session-info">
              <span class="session-name">{{ s.connection_name }}</span>
              <span class="session-time">{{ s.suspended_at }}</span>
            </div>
            <div class="session-actions">
              <button class="btn-sm" @click="resume(s.id)">恢复</button>
              <button class="btn-sm danger" @click="terminate(s.id)">终止</button>
            </div>
          </div>
          <div v-if="!sessions.length" class="empty">暂无挂起会话</div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { sshSuspendApi, type SuspendedSession } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{ visible: boolean }>();
defineEmits<{ close: [] }>();
const notify = useUINotificationStore();
const sessions = ref<SuspendedSession[]>([]);

watch(() => props.visible, async (v) => {
  if (v) {
    try { sessions.value = await sshSuspendApi.list(); } catch { sessions.value = []; }
  }
});

async function resume(id: string) {
  try {
    await sshSuspendApi.resume(id);
    sessions.value = sessions.value.filter(s => s.id !== id);
    notify.addNotification('success', '会话已恢复');
  } catch (e: any) { notify.addNotification('error', e.message); }
}

async function terminate(id: string) {
  try {
    await sshSuspendApi.terminate(id);
    sessions.value = sessions.value.filter(s => s.id !== id);
    notify.addNotification('success', '会话已终止');
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.modal-card { background: var(--bg-surface0); border-radius: 8px; padding: 20px; width: 480px; max-height: 70vh; overflow-y: auto; border: 1px solid var(--border); }
.modal-header { display: flex; justify-content: space-between; align-items: center; font-size: 16px; font-weight: 600; margin-bottom: 16px; }
.close-btn { cursor: pointer; font-size: 20px; color: var(--text-dim); }
.close-btn:hover { color: var(--red); }
.session-list { display: flex; flex-direction: column; gap: 4px; }
.session-item { display: flex; justify-content: space-between; align-items: center; padding: 8px; border-radius: 4px; background: var(--bg-mantle); }
.session-info { display: flex; flex-direction: column; gap: 2px; }
.session-name { font-size: 13px; }
.session-time { font-size: 11px; color: var(--text-dim); }
.session-actions { display: flex; gap: 4px; }
.btn-sm { padding: 3px 10px; border-radius: 3px; border: 1px solid var(--border); background: transparent; color: var(--text); cursor: pointer; font-size: 12px; }
.btn-sm:hover { background: var(--bg-surface1); }
.btn-sm.danger { color: var(--red); border-color: var(--red); }
.btn-sm.danger:hover { background: rgba(243,139,168,0.1); }
.empty { text-align: center; color: var(--text-dim); font-size: 13px; padding: 16px; }
</style>

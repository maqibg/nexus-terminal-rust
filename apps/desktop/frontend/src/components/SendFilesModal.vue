<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('cancel')">
      <div class="form-card">
        <div class="form-title">发送文件到远程服务器</div>
        <div class="form-body">
          <label class="field-label">源文件路径 (当前 SFTP)</label>
          <input class="input" v-model="srcPath" placeholder="/path/to/file" />

          <label class="field-label">目标连接</label>
          <AppSelect
            v-model="targetConnectionId"
            :options="targetConnectionOptions"
            variant="input"
            aria-label="目标连接"
          />

          <label class="field-label">目标路径</label>
          <input class="input" v-model="destPath" placeholder="/remote/path/filename" />
        </div>
        <div v-if="status" class="status" :class="statusType">{{ status }}</div>
        <div class="form-actions">
          <button class="btn btn-cancel" @click="$emit('cancel')">取消</button>
          <button class="btn btn-primary" @click="send" :disabled="sending || !srcPath || !targetConnectionId || !destPath">
            {{ sending ? '发送中...' : '发送' }}
          </button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { computed, ref, onMounted } from 'vue';
import AppSelect from './AppSelect.vue';
import { connectionsApi, transferApi, type Connection } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{ visible: boolean; sessionId: string; currentFile?: string }>();
const emit = defineEmits<{ cancel: []; sent: [taskId: string] }>();
const notify = useUINotificationStore();

const connections = ref<Connection[]>([]);
const srcPath = ref('');
const targetConnectionId = ref(0);
const destPath = ref('/');
const sending = ref(false);
const status = ref('');
const statusType = ref<'info' | 'success' | 'error'>('info');

const targetConnections = computed(() =>
  connections.value.filter((conn) => String(conn.type ?? 'SSH').toUpperCase() === 'SSH')
);

const targetConnectionOptions = computed(() => [
  { value: 0, label: '选择目标连接', disabled: true },
  ...targetConnections.value.map(connection => ({
    value: connection.id,
    label: `${connection.name} (${connection.host})`,
  })),
]);

onMounted(async () => {
  connections.value = await connectionsApi.list();
  if (props.currentFile) {
    srcPath.value = props.currentFile;
    const fileName = props.currentFile.split('/').pop();
    if (fileName) {
      destPath.value = `/${fileName}`;
    }
  }
});

async function send() {
  sending.value = true;
  status.value = '';
  try {
    status.value = '正在发送文件，请稍候...';
    statusType.value = 'info';

    const taskId = await transferApi.send(
      props.sessionId,
      targetConnectionId.value,
      srcPath.value,
      destPath.value,
    );

    status.value = '任务已创建，可在传输进度中查看实时状态';
    statusType.value = 'success';
    notify.addNotification('success', '已创建跨连接传输任务');
    emit('sent', taskId);
  } catch (e: any) {
    status.value = `发送失败: ${e.message}`;
    statusType.value = 'error';
    notify.addNotification('error', e.message);
  } finally {
    sending.value = false;
  }
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.form-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; width: 440px; border: 1px solid var(--border); }
.form-title { font-size: calc(16px + var(--ui-font-size-offset)); font-weight: 600; margin-bottom: 16px; }
.form-body { display: flex; flex-direction: column; gap: 6px; margin-bottom: 12px; }
.field-label { font-size: calc(12px + var(--ui-font-size-offset)); color: var(--text-sub); margin-top: 4px; }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 8px; color: var(--text); font-size: calc(13px + var(--ui-font-size-offset)); outline: none; }
.input:focus { border-color: var(--blue); box-shadow: 0 0 0 1px var(--blue); }
.status { font-size: calc(12px + var(--ui-font-size-offset)); padding: 6px 8px; border-radius: 4px; margin-bottom: 8px; }
.status.info { color: var(--blue); background: rgba(137,180,250,0.1); }
.status.success { color: var(--green); background: rgba(166,227,161,0.1); }
.status.error { color: var(--red); background: rgba(243,139,168,0.1); }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-primary { background: var(--blue); color: var(--bg-base); font-weight: 600; }
.btn:disabled { opacity: 0.4; cursor: default; }
</style>

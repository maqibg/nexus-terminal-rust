<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="cancel">
      <div class="upload-card">
        <div class="upload-title">上传文件</div>
        <div
          class="drop-zone"
          :class="{ dragging }"
          @dragover.prevent="dragging = true"
          @dragleave="dragging = false"
          @drop.prevent="onDrop"
          @click="pickFiles"
        >
          <span v-if="!uploading">拖拽文件到此处，或点击选择本地文件</span>
          <span v-else>正在创建上传任务... {{ progress }}%</span>
        </div>
        <div v-if="uploading" class="progress-bar">
          <div class="progress-fill" :style="{ width: progress + '%' }"></div>
        </div>
        <div class="upload-actions">
          <button class="btn btn-cancel" @click="cancel" :disabled="uploading">取消</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { sftpApi } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{
  visible: boolean;
  sessionId: string;
  remotePath: string;
}>();
const emit = defineEmits<{ uploaded: []; cancel: [] }>();

const notify = useUINotificationStore();
const dragging = ref(false);
const uploading = ref(false);
const progress = ref(0);

function cancel() { if (!uploading.value) emit('cancel'); }

function normalizeRemotePath(basePath: string, fileName: string): string {
  return basePath.endsWith('/') ? `${basePath}${fileName}` : `${basePath}/${fileName}`;
}

function fileNameFromPath(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || 'file';
}

function announceTransfer(taskId: string) {
  window.dispatchEvent(new CustomEvent('transfer-created', { detail: { taskId } }));
}

async function pickFiles() {
  if (uploading.value) return;
  const selected = await open({ multiple: true, directory: false });
  if (!selected) return;
  const paths = Array.isArray(selected) ? selected : [selected];
  await uploadFromPaths(paths);
}

function onDrop(e: DragEvent) {
  dragging.value = false;
  const files = e.dataTransfer?.files;
  if (!files?.length) return;

  const paths = Array.from(files)
    .map((file) => (file as File & { path?: string }).path)
    .filter((v): v is string => !!v);

  if (!paths.length) {
    notify.addNotification('error', '拖拽上传仅支持带本地路径的桌面文件');
    return;
  }

  uploadFromPaths(paths);
}

async function uploadFromPaths(paths: string[]) {
  uploading.value = true;
  progress.value = 0;

  try {
    const total = paths.length;
    let finished = 0;

    for (const localPath of paths) {
      const fileName = fileNameFromPath(localPath);
      const remotePath = normalizeRemotePath(props.remotePath, fileName);
      const taskId = await sftpApi.uploadFromDisk(props.sessionId, localPath, remotePath);
      announceTransfer(taskId);

      finished += 1;
      progress.value = Math.round((finished / total) * 100);
    }

    notify.addNotification('success', `已创建 ${paths.length} 个上传任务`);
    emit('uploaded');
  } catch (e: any) {
    notify.addNotification('error', `上传失败: ${e.message}`);
  } finally {
    uploading.value = false;
  }
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.upload-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; min-width: 400px; border: 1px solid var(--border); }
.upload-title { font-size: 16px; font-weight: 600; margin-bottom: 16px; }
.drop-zone { border: 2px dashed var(--border); border-radius: 6px; padding: 32px; text-align: center; color: var(--text-dim); cursor: pointer; font-size: 13px; }
.drop-zone.dragging { border-color: var(--blue); color: var(--blue); }
.drop-zone:hover { border-color: var(--text-dim); }
.progress-bar { margin-top: 12px; height: 4px; background: var(--bg-surface1); border-radius: 2px; overflow: hidden; }
.progress-fill { height: 100%; background: var(--blue); transition: width 0.2s; }
.upload-actions { display: flex; justify-content: flex-end; margin-top: 16px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn:disabled { opacity: 0.4; }
</style>

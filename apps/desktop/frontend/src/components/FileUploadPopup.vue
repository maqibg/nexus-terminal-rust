<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="cancel">
      <div class="upload-card">
        <div class="upload-title">上传文件 / 文件夹</div>
        <div
          class="drop-zone"
          :class="{ dragging }"
          @dragover.prevent="dragging = true"
          @dragleave="dragging = false"
          @drop.prevent="onDrop"
          @click="pickFiles"
        >
          <span v-if="!uploading">拖拽文件或文件夹到此处，或点击选择本地文件</span>
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
import { onMounted, onUnmounted, ref } from 'vue';
import { open } from '@tauri-apps/plugin-dialog';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { DragDropEvent as TauriDragDropEvent } from '@tauri-apps/api/webview';
import { createUploadTasksFromLocalPaths } from '@/lib/local-upload';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{
  visible: boolean;
  sessionId: string;
  remotePath: string;
}>();
const emit = defineEmits<{ uploaded: [taskIds: string[]]; cancel: [] }>();

const notify = useUINotificationStore();
const dragging = ref(false);
const uploading = ref(false);
const progress = ref(0);
let unlistenWindowDragDrop: (() => void) | null = null;

function cancel() { if (!uploading.value) emit('cancel'); }

function normalizeRemotePath(basePath: string, fileName: string): string {
  return basePath.endsWith('/') ? `${basePath}${fileName}` : `${basePath}/${fileName}`;
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

  if (paths.length) {
    void uploadFromPaths(paths);
  }
}

async function uploadFromPaths(paths: string[]) {
  uploading.value = true;
  progress.value = 0;

  try {
    const { taskIds, uploadEntries } = await createUploadTasksFromLocalPaths({
      sessionId: props.sessionId,
      paths,
      remoteBasePath: props.remotePath,
      joinRemotePath: normalizeRemotePath,
      onTaskCreated: (taskId) => {
        announceTransfer(taskId);
      },
    });

    if (!taskIds.length) {
      notify.addNotification('warning', '未找到可上传的文件');
      return;
    }

    progress.value = uploadEntries.length > 0 ? 100 : 0;
    notify.addNotification('success', `已创建 ${taskIds.length} 个上传任务`);
    emit('uploaded', taskIds);
  } catch (e: any) {
    notify.addNotification('error', `上传失败: ${e.message}`);
  } finally {
    uploading.value = false;
  }
}

async function handleWindowDragDropEvent(event: TauriDragDropEvent): Promise<void> {
  if (event.type === 'leave') {
    dragging.value = false;
    return;
  }

  if (event.type !== 'drop' || !props.visible || uploading.value) {
    return;
  }

  dragging.value = false;
  const paths = event.paths ?? [];
  if (!paths.length) {
    return;
  }

  await uploadFromPaths(paths);
}

onMounted(() => {
  void getCurrentWindow()
    .onDragDropEvent((event) => {
      void handleWindowDragDropEvent(event.payload);
    })
    .then((unlisten) => {
      unlistenWindowDragDrop = unlisten;
    });
});

onUnmounted(() => {
  unlistenWindowDragDrop?.();
  unlistenWindowDragDrop = null;
});
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

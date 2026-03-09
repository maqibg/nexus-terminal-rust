import { ref, onUnmounted } from 'vue';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { transferApi, sftpApi, type TransferTask, type TransferTaskDto } from '@/lib/api';

interface TransferProgressEvent {
  task_id: string;
  file_name: string;
  kind?: 'upload' | 'download';
  bytes_transferred: number;
  total_bytes: number;
  percent: number;
}

interface TransferStatusEvent {
  task_id: string;
  file_name?: string;
  kind?: 'upload' | 'download';
  status: 'active' | 'paused' | 'completed' | 'failed' | 'cancelled';
  error?: string | null;
}

const statusMap: Record<TransferTaskDto['status'], TransferTask['status']> = {
  Pending: 'active',
  InProgress: 'active',
  Paused: 'paused',
  Completed: 'completed',
  Failed: 'failed',
  Cancelled: 'cancelled',
};

function fromDto(dto: TransferTaskDto): TransferTask {
  const kind = dto.kind;
  const percent = dto.total_bytes > 0
    ? Math.min(100, Math.round((dto.transferred_bytes / dto.total_bytes) * 100))
    : 0;

  return {
    id: dto.id,
    kind,
    fileName: dto.file_name,
    totalBytes: dto.total_bytes,
    transferredBytes: dto.transferred_bytes,
    percent,
    status: statusMap[dto.status] ?? 'active',
  };
}

export function useTransferProgress() {
  const tasks = ref<Map<string, TransferTask>>(new Map());
  const taskList = ref<TransferTask[]>([]);
  let progressUnlisten: UnlistenFn | null = null;
  let statusUnlisten: UnlistenFn | null = null;
  let pollTimer: ReturnType<typeof setInterval> | null = null;
  let isListening = false;

  function updateList() {
    taskList.value = Array.from(tasks.value.values()).sort((a, b) => {
      const aDone = a.status === 'completed' || a.status === 'failed' || a.status === 'cancelled';
      const bDone = b.status === 'completed' || b.status === 'failed' || b.status === 'cancelled';
      if (aDone === bDone) return 0;
      return aDone ? 1 : -1;
    });
  }

  function upsertFromProgress(payload: TransferProgressEvent) {
    const existing = tasks.value.get(payload.task_id);
    const next: TransferTask = existing ?? {
      id: payload.task_id,
      kind: payload.kind === 'upload' ? 'upload' : 'download',
      fileName: payload.file_name,
      totalBytes: payload.total_bytes,
      transferredBytes: payload.bytes_transferred,
      percent: payload.percent,
      status: 'active',
    };

    next.fileName = payload.file_name;
    next.kind = payload.kind === 'upload' ? 'upload' : next.kind;
    next.totalBytes = payload.total_bytes;
    next.transferredBytes = payload.bytes_transferred;
    next.percent = payload.percent;
    if (payload.percent >= 100 && next.status === 'active') {
      next.status = 'completed';
    }

    tasks.value.set(payload.task_id, next);
    updateList();
  }

  function upsertFromStatus(payload: TransferStatusEvent) {
    const existing = tasks.value.get(payload.task_id);
    const next: TransferTask = existing ?? {
      id: payload.task_id,
      kind: payload.kind === 'upload' ? 'upload' : 'download',
      fileName: payload.file_name ?? payload.task_id,
      totalBytes: 0,
      transferredBytes: 0,
      percent: 0,
      status: 'active',
    };

    if (payload.file_name) next.fileName = payload.file_name;
    if (payload.kind) next.kind = payload.kind;
    next.status = payload.status;
    if (payload.status === 'completed') next.percent = 100;

    tasks.value.set(payload.task_id, next);
    updateList();
  }

  async function syncFromBackend() {
    try {
      const list = await transferApi.list();
      for (const item of list) {
        tasks.value.set(item.id, fromDto(item));
      }
      updateList();
    } catch {
      // ignore sync errors to keep UI responsive
    }
  }

  async function startListening() {
    if (isListening) return;
    isListening = true;

    try {
      await syncFromBackend();

      progressUnlisten = await listen<TransferProgressEvent>('transfers/progress', (e) => {
        upsertFromProgress(e.payload);
      });

      statusUnlisten = await listen<TransferStatusEvent>('transfers/status', (e) => {
        upsertFromStatus(e.payload);
      });

      if (!pollTimer) {
        pollTimer = setInterval(() => {
          void syncFromBackend();
        }, 1500);
      }
    } catch (error) {
      cleanup();
      throw error;
    }
  }

  async function cancelTask(taskId: string) {
    try {
      await transferApi.cancel(taskId);
    } catch {
      try {
        await sftpApi.cancelTask(taskId);
      } catch {
        // ignore
      }
    }

    const task = tasks.value.get(taskId);
    if (task) {
      task.status = 'cancelled';
      updateList();
    }
  }

  async function pauseAll(): Promise<void> {
    await transferApi.pauseAll();
    for (const task of tasks.value.values()) {
      if (task.status === 'active') {
        task.status = 'paused';
      }
    }
    updateList();
  }

  async function resumeAll(): Promise<void> {
    await transferApi.resumeAll();
    for (const task of tasks.value.values()) {
      if (task.status === 'paused') {
        task.status = task.percent >= 100 ? 'completed' : 'active';
      }
    }
    updateList();
  }

  async function cancelAll(): Promise<void> {
    await transferApi.cancelAll();
    for (const task of tasks.value.values()) {
      if (task.status === 'active' || task.status === 'paused') {
        task.status = 'cancelled';
      }
    }
    updateList();
  }

  async function clearCompleted(): Promise<void> {
    await transferApi.cleanupCompleted();
    const next = new Map<string, TransferTask>();
    for (const [id, task] of tasks.value.entries()) {
      if (task.status === 'active' || task.status === 'paused') {
        next.set(id, task);
      }
    }
    tasks.value = next;
    updateList();
    await syncFromBackend();
  }

  function cleanup() {
    progressUnlisten?.();
    statusUnlisten?.();
    progressUnlisten = null;
    statusUnlisten = null;
    isListening = false;

    if (pollTimer) {
      clearInterval(pollTimer);
      pollTimer = null;
    }
  }

  onUnmounted(cleanup);

  return {
    taskList,
    startListening,
    cancelTask,
    pauseAll,
    resumeAll,
    cancelAll,
    clearCompleted,
    cleanup,
    syncFromBackend,
  };
}

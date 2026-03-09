import { tauriInvoke } from './invoke';

/** UI-facing transfer task state (used by TransferProgressModal & useTransferProgress). */
export interface TransferTask {
  id: string;
  kind: 'upload' | 'download';
  fileName: string;
  totalBytes: number;
  transferredBytes: number;
  percent: number;
  status: 'active' | 'paused' | 'completed' | 'failed' | 'cancelled';
}

export interface TransferTaskDto {
  id: string;
  kind: 'upload' | 'download';
  file_name: string;
  total_bytes: number;
  transferred_bytes: number;
  status: 'Pending' | 'InProgress' | 'Paused' | 'Completed' | 'Failed' | 'Cancelled';
  error?: string | null;
}

export const transferApi = {
  send: (sourceSessionId: string, targetConnectionId: number, sourcePath: string, targetPath: string) =>
    tauriInvoke<string>('transfer_send', {
      req: {
        source_session_id: sourceSessionId,
        target_connection_id: targetConnectionId,
        source_path: sourcePath,
        target_path: targetPath,
      },
    }),

  list: () => tauriInvoke<TransferTaskDto[]>('transfer_list'),

  get: (taskId: string) =>
    tauriInvoke<TransferTaskDto | null>('transfer_get', {
      req: { task_id: taskId },
    }),

  cancel: (taskId: string) =>
    tauriInvoke<void>('transfer_cancel', {
      req: { task_id: taskId },
    }),

  pause: (taskId: string) =>
    tauriInvoke<void>('transfer_pause', {
      req: { task_id: taskId },
    }),

  resume: (taskId: string) =>
    tauriInvoke<void>('transfer_resume', {
      req: { task_id: taskId },
    }),

  pauseAll: () =>
    tauriInvoke<void>('transfer_pause_all'),

  resumeAll: () =>
    tauriInvoke<void>('transfer_resume_all'),

  cancelAll: () =>
    tauriInvoke<void>('transfer_cancel_all'),

  cleanupCompleted: () =>
    tauriInvoke<void>('transfer_cleanup_completed'),
};

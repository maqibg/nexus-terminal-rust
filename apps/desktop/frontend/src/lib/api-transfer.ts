import { tauriInvoke } from './invoke';

export interface TransferTaskDto {
  id: string;
  kind: 'Upload' | 'Download';
  file_name: string;
  total_bytes: number;
  transferred_bytes: number;
  status: 'Pending' | 'InProgress' | 'Completed' | 'Failed' | 'Cancelled';
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
};

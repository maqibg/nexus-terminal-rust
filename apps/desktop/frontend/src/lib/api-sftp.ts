/**
 * SFTP API — replaces WebSocket-based SFTP transport.
 */
import { tauriInvoke } from './invoke';

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  modified?: number;
  permissions?: number;
}

export const sftpApi = {
  open: (connectionId: number) =>
    tauriInvoke<string>('sftp_open', {
      req: { connection_id: connectionId },
    }),

  close: (sessionId: string) =>
    tauriInvoke<void>('sftp_close', {
      req: { session_id: sessionId },
    }),

  listDir: (sessionId: string, path: string) =>
    tauriInvoke<FileEntry[]>('sftp_list_dir', {
      req: { session_id: sessionId, path },
    }),

  readFile: (sessionId: string, path: string) =>
    tauriInvoke<string>('sftp_read_file', {
      req: { session_id: sessionId, path },
    }),

  writeFile: (sessionId: string, path: string, data: string) =>
    tauriInvoke<void>('sftp_write_file', {
      req: { session_id: sessionId, path, data },
    }),

  mkdir: (sessionId: string, path: string) =>
    tauriInvoke<void>('sftp_mkdir', {
      req: { session_id: sessionId, path },
    }),

  rmdir: (sessionId: string, path: string) =>
    tauriInvoke<void>('sftp_rmdir', {
      req: { session_id: sessionId, path },
    }),

  removeFile: (sessionId: string, path: string) =>
    tauriInvoke<void>('sftp_remove_file', {
      req: { session_id: sessionId, path },
    }),

  rename: (sessionId: string, oldPath: string, newPath: string) =>
    tauriInvoke<void>('sftp_rename', {
      req: { session_id: sessionId, old_path: oldPath, new_path: newPath },
    }),

  stat: (sessionId: string, path: string) =>
    tauriInvoke<FileEntry>('sftp_stat', {
      req: { session_id: sessionId, path },
    }),

  chmod: (sessionId: string, path: string, mode: number) =>
    tauriInvoke<void>('sftp_chmod', {
      req: { session_id: sessionId, path, mode },
    }),

  uploadChunk: (sessionId: string, path: string, chunkIndex: number, dataBase64: string) =>
    tauriInvoke<void>('sftp_upload_chunk', {
      req: { session_id: sessionId, path, chunk_index: chunkIndex, data_base64: dataBase64 },
    }),

  downloadFile: (sessionId: string, path: string) =>
    tauriInvoke<string>('sftp_download_file', {
      req: { session_id: sessionId, path },
    }),

  downloadToDisk: (sessionId: string, remotePath: string, localPath: string) =>
    tauriInvoke<string>('sftp_download_to_disk', {
      req: { session_id: sessionId, remote_path: remotePath, local_path: localPath },
    }),

  uploadFromDisk: (sessionId: string, localPath: string, remotePath: string) =>
    tauriInvoke<string>('sftp_upload_from_disk', {
      req: { session_id: sessionId, local_path: localPath, remote_path: remotePath },
    }),

  downloadDirectoryToDisk: (sessionId: string, remotePath: string, localZipPath: string) =>
    tauriInvoke<string>('sftp_download_directory_to_disk', {
      req: { session_id: sessionId, remote_path: remotePath, local_zip_path: localZipPath },
    }),

  cancelTask: (taskId: string) =>
    tauriInvoke<void>('sftp_cancel_task', {
      req: { task_id: taskId },
    }),
};

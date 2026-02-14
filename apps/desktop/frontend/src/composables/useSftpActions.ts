import type { Ref } from 'vue';
import { sftpApi, type FileEntry } from '@/lib/api';
import { useAlertDialog } from './useAlertDialog';

/**
 * SFTP operations composable — wraps api-sftp with error handling.
 */
export function useSftpActions(sessionId: Ref<string>) {
  const { alert } = useAlertDialog();

  async function withError<T>(fn: () => Promise<T>): Promise<T | undefined> {
    try {
      return await fn();
    } catch (e: any) {
      await alert('SFTP Error', e.message ?? String(e));
      return undefined;
    }
  }

  const listDir = (path: string) =>
    withError(() => sftpApi.listDir(sessionId.value, path)) as Promise<FileEntry[] | undefined>;

  const readFile = (path: string) =>
    withError(() => sftpApi.readFile(sessionId.value, path));

  const writeFile = (path: string, content: string) =>
    withError(() => sftpApi.writeFile(sessionId.value, path, content));

  const mkdir = (path: string) =>
    withError(() => sftpApi.mkdir(sessionId.value, path));

  const rmdir = (path: string) =>
    withError(() => sftpApi.rmdir(sessionId.value, path));

  const removeFile = (path: string) =>
    withError(() => sftpApi.removeFile(sessionId.value, path));

  const rename = (oldPath: string, newPath: string) =>
    withError(() => sftpApi.rename(sessionId.value, oldPath, newPath));

  const chmod = (path: string, mode: number) =>
    withError(() => sftpApi.chmod(sessionId.value, path, mode));

  const stat = (path: string) =>
    withError(() => sftpApi.stat(sessionId.value, path));

  return { listDir, readFile, writeFile, mkdir, rmdir, removeFile, rename, chmod, stat };
}

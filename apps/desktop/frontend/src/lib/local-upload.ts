import { sftpApi, type LocalUploadEntry } from './api-sftp';

const TASK_CREATE_BATCH_SIZE = 8;

export interface CreateUploadTasksOptions {
  sessionId: string;
  paths: string[];
  remoteBasePath: string;
  joinRemotePath: (basePath: string, childName: string) => string;
  onTaskCreated?: (taskId: string) => void;
  batchSize?: number;
}

export interface CreateUploadTasksResult {
  taskIds: string[];
  uploadEntries: LocalUploadEntry[];
}

export function fileNameFromLocalPath(path: string): string {
  const parts = path.split(/[\\/]/);
  return parts[parts.length - 1] || 'file';
}

function normalizeRelativePath(relativePath: string): string[] {
  return relativePath
    .replace(/\\/g, '/')
    .split('/')
    .map((segment) => segment.trim())
    .filter(Boolean);
}

export function buildRemoteUploadPath(
  remoteBasePath: string,
  entry: LocalUploadEntry,
  joinRemotePath: (basePath: string, childName: string) => string,
): string {
  let remotePath = remoteBasePath;
  for (const segment of normalizeRelativePath(entry.relative_path)) {
    remotePath = joinRemotePath(remotePath, segment);
  }
  return joinRemotePath(remotePath, fileNameFromLocalPath(entry.local_path));
}

function chunkEntries(entries: LocalUploadEntry[], batchSize: number): LocalUploadEntry[][] {
  const chunks: LocalUploadEntry[][] = [];
  for (let i = 0; i < entries.length; i += batchSize) {
    chunks.push(entries.slice(i, i + batchSize));
  }
  return chunks;
}

export async function createUploadTasksFromLocalPaths(
  options: CreateUploadTasksOptions,
): Promise<CreateUploadTasksResult> {
  const batchSize = Math.max(1, options.batchSize ?? TASK_CREATE_BATCH_SIZE);
  const uploadEntries = await sftpApi.collectLocalUploadEntries(options.paths);
  const taskIds: string[] = [];

  for (const batch of chunkEntries(uploadEntries, batchSize)) {
    const createdIds = await Promise.all(
      batch.map(async (entry) => {
        const remotePath = buildRemoteUploadPath(
          options.remoteBasePath,
          entry,
          options.joinRemotePath,
        );
        const taskId = await sftpApi.uploadFromDisk(
          options.sessionId,
          entry.local_path,
          remotePath,
        );
        options.onTaskCreated?.(taskId);
        return taskId;
      }),
    );
    taskIds.push(...createdIds);
  }

  return { taskIds, uploadEntries };
}

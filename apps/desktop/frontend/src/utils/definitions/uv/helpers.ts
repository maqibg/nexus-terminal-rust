
import type { CompletionContext, CompletionItem } from '../../types';
import { getRemoteFiles } from '../../providers/file-system';

export const UV_ROOT_PATH_VALUE_OPTIONS = new Set(['--directory', '--project', '--config-file', '--cache-dir']);

export async function suggestRemotePaths(ctx: CompletionContext, fallback = './'): Promise<CompletionItem[]> {
    if (!ctx.sessionId || !ctx.electronAPI) return [];
    return getRemoteFiles(ctx.sessionId, ctx.currentArg || fallback, ctx.electronAPI);
}



import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * touch 命令 - 创建文件或更新时间戳
 */
const touchCommand: CommandDefinition = {
    name: 'touch',
    description: '创建空文件或更新时间戳',
    options: [
        { text: '-a', type: 'option', description: '只更新访问时间', priority: 95 },
        { text: '-m', type: 'option', description: '只更新修改时间', priority: 95 },
        { text: '-c', type: 'option', description: '不创建新文件', priority: 90 },
        { text: '--no-create', type: 'option', description: '不创建新文件', priority: 90 },
        { text: '-d', type: 'option', description: '指定时间', priority: 85, usage: `-d "2026-03-11 10:00:00"` },
        { text: '-t', type: 'option', description: '指定时间戳', priority: 80, usage: '-t 202603111000' },
        { text: '-r', type: 'option', description: '参考文件时间', priority: 75, usage: '-r ref.txt' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-r') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    }
};

export default touchCommand;


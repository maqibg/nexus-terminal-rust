
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * uniq 命令定义
 */
const uniqCommand: CommandDefinition = {
    name: 'uniq',
    description: '相邻行去重/统计',
    options: [
        { text: '-c', type: 'option', description: '显示计数', priority: 95, usage: 'uniq -c' },
        { text: '-d', type: 'option', description: '只显示重复行', priority: 90 },
        { text: '-u', type: 'option', description: '只显示唯一行', priority: 85 },
        { text: '-i', type: 'option', description: '忽略大小写', priority: 80 },
        { text: '-f', type: 'option', description: '跳过前 N 字段', priority: 70, usage: '-f 1' },
        { text: '-s', type: 'option', description: '跳过前 N 字符', priority: 65, usage: '-s 10' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (!ctx.currentArg.startsWith('-') && prev !== '-f' && prev !== '-s') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    }
};

export default uniqCommand;


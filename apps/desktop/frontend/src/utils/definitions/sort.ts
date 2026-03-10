
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * sort 命令定义
 */
const sortCommand: CommandDefinition = {
    name: 'sort',
    description: '排序文本',
    options: [
        { text: '-n', type: 'option', description: '按数字排序', priority: 95 },
        { text: '-r', type: 'option', description: '倒序', priority: 90 },
        { text: '-u', type: 'option', description: '去重', priority: 85 },
        { text: '-h', type: 'option', description: '按人类可读数值', priority: 80 },
        { text: '-V', type: 'option', description: '按版本号', priority: 75 },
        { text: '-k', type: 'option', description: '指定 key', priority: 70, usage: '-k 1,1' },
        { text: '-t', type: 'option', description: '分隔符', priority: 65, usage: `-t ','` },
        { text: '-S', type: 'option', description: '缓冲区大小', priority: 60, usage: '-S 50%' },
        { text: '-o', type: 'option', description: '输出到文件', priority: 55, usage: '-o out.txt' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-o') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-') && prev !== '-k' && prev !== '-t' && prev !== '-S') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    }
};

export default sortCommand;


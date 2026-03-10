
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * sed 命令定义
 */
const sedCommand: CommandDefinition = {
    name: 'sed',
    description: '流编辑器',
    options: [
        { text: '-n', type: 'option', description: '安静模式(只打印 p)', priority: 90 },
        { text: '-i', type: 'option', description: '原地修改', priority: 85, usage: `-i 's/a/b/g' file` },
        { text: '-E', type: 'option', description: '扩展正则', priority: 80 },
        { text: '-r', type: 'option', description: '扩展正则', priority: 80 },
        { text: '-e', type: 'option', description: '脚本', priority: 75, usage: `-e 's/a/b/g'` },
        { text: '-f', type: 'option', description: '脚本文件', priority: 70, usage: '-f script.sed' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-f') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-') && prev !== '-e') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    }
};

export default sedCommand;


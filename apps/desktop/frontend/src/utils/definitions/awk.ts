
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * awk 命令定义
 */
const awkCommand: CommandDefinition = {
    name: 'awk',
    description: '文本处理工具',
    options: [
        { text: '-F', type: 'option', description: '字段分隔符', priority: 90, usage: `-F ':' '{print $1}'` },
        { text: '-v', type: 'option', description: '变量', priority: 85, usage: `-v k=v '{print k}'` },
        { text: '-f', type: 'option', description: '脚本文件', priority: 80, usage: '-f script.awk' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-f') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-') && prev !== '-v' && prev !== '-F') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    }
};

export default awkCommand;


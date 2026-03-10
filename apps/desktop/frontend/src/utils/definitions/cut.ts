
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * cut 命令定义
 */
const cutCommand: CommandDefinition = {
    name: 'cut',
    description: '按字段/字符切分文本',
    options: [
        { text: '-d', type: 'option', description: '分隔符', priority: 95, usage: `-d ':'` },
        { text: '-f', type: 'option', description: '字段范围', priority: 90, usage: '-f 1,3' },
        { text: '-c', type: 'option', description: '字符范围', priority: 85, usage: '-c 1-10' },
        { text: '-b', type: 'option', description: '字节范围', priority: 80, usage: '-b 1-10' },
        { text: '--complement', type: 'option', description: '取反', priority: 70 },
        { text: '-s', type: 'option', description: '仅输出含分隔符行', priority: 65 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    }
};

export default cutCommand;


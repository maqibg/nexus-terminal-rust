
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteDirectories } from '../providers/file-system';

/**
 * lsof 命令定义
 */
const lsofCommand: CommandDefinition = {
    name: 'lsof',
    description: '列出打开文件',
    options: [
        { text: '-i', type: 'option', description: '网络连接', priority: 95, usage: '-i :80' },
        { text: '-p', type: 'option', description: '按 PID 过滤', priority: 90, usage: '-p 1234' },
        { text: '-u', type: 'option', description: '按用户过滤', priority: 85, usage: '-u root' },
        { text: '-n', type: 'option', description: '不解析主机名', priority: 80 },
        { text: '-P', type: 'option', description: '不解析端口', priority: 75 },
        { text: '-t', type: 'option', description: '仅输出 PID', priority: 70 },
        { text: '+D', type: 'option', description: '递归目录', priority: 65, usage: '+D /path' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '+D') {
            return getRemoteDirectories(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        if (prev === '-i' && !ctx.currentArg) {
            return [
                { text: ':80', type: 'hint' as const, description: '端口 80', priority: 80, matchPart: '', restPart: ':80' },
                { text: 'TCP:22', type: 'hint' as const, description: 'TCP 22', priority: 78, matchPart: '', restPart: 'TCP:22' },
            ];
        }
        return [];
    }
};

export default lsofCommand;


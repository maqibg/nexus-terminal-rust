
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * Node.js 命令定义
 */

const NODE_PRELOAD_HINTS: CompletionItem[] = [
    { text: 'dotenv/config', type: 'subcommand', description: '加载 .env (require preload)', priority: 95, matchPart: '', restPart: 'dotenv/config' },
    { text: 'ts-node/register', type: 'subcommand', description: 'TypeScript 运行时', priority: 90, matchPart: '', restPart: 'ts-node/register' },
    { text: 'source-map-support/register', type: 'subcommand', description: '更友好的堆栈信息', priority: 85, matchPart: '', restPart: 'source-map-support/register' },
];

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

const nodeCommand: CommandDefinition = {
    name: 'node',
    description: 'Node.js 运行时',
    options: [
        { text: '-e', type: 'option', description: '执行脚本字符串', priority: 100, usage: 'node -e "console.log(1)"' },
        { text: '-p', type: 'option', description: '执行并打印结果', priority: 95, usage: 'node -p "1+2"' },
        { text: '-r', type: 'option', description: '预加载模块', priority: 90, usage: 'node -r dotenv/config app.js', repeatable: true },
        { text: '--inspect', type: 'option', description: '启用调试器', priority: 85 },
        { text: '--inspect-brk', type: 'option', description: '启动即断点', priority: 80 },
        { text: '--trace-warnings', type: 'option', description: '打印警告堆栈', priority: 75 },
        { text: '--no-warnings', type: 'option', description: '禁用警告', priority: 70 },
        { text: '-v', type: 'option', description: '显示版本', priority: 60 },
        { text: '--version', type: 'option', description: '显示版本', priority: 60 },
        { text: '-h', type: 'option', description: '显示帮助', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 55 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-r') {
            return filterHints(NODE_PRELOAD_HINTS, ctx.currentArg).map((item) => ({
                ...item,
                matchPart: ctx.currentArg,
                restPart: item.text.substring(ctx.currentArg.length),
            }));
        }

        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    },
};

export default nodeCommand;


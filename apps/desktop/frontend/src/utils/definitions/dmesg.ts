
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * dmesg 命令定义
 */
const LEVEL_HINTS: CompletionItem[] = [
    { text: 'emerg', type: 'subcommand', description: '0', priority: 90, matchPart: '', restPart: 'emerg' },
    { text: 'alert', type: 'subcommand', description: '1', priority: 89, matchPart: '', restPart: 'alert' },
    { text: 'crit', type: 'subcommand', description: '2', priority: 88, matchPart: '', restPart: 'crit' },
    { text: 'err', type: 'subcommand', description: '3', priority: 87, matchPart: '', restPart: 'err' },
    { text: 'warn', type: 'subcommand', description: '4', priority: 86, matchPart: '', restPart: 'warn' },
    { text: 'notice', type: 'subcommand', description: '5', priority: 85, matchPart: '', restPart: 'notice' },
    { text: 'info', type: 'subcommand', description: '6', priority: 84, matchPart: '', restPart: 'info' },
    { text: 'debug', type: 'subcommand', description: '7', priority: 83, matchPart: '', restPart: 'debug' },
];

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

const dmesgCommand: CommandDefinition = {
    name: 'dmesg',
    description: '内核环形缓冲区日志',
    options: [
        { text: '-T', type: 'option', description: '显示可读时间', priority: 95 },
        { text: '-H', type: 'option', description: '人类可读输出', priority: 90 },
        { text: '-w', type: 'option', description: '实时跟踪', priority: 85 },
        { text: '-k', type: 'option', description: '只显示内核消息', priority: 80 },
        { text: '-l', type: 'option', description: '按级别过滤', priority: 75, usage: '-l err,warn' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-l') {
            return filterHints(LEVEL_HINTS, ctx.currentArg);
        }
        return [];
    }
};

export default dmesgCommand;


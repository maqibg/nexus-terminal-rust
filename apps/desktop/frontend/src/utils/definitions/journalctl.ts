
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * journalctl 命令定义
 */

const OUTPUT_FORMAT_HINTS: CompletionItem[] = [
    { text: 'short', type: 'subcommand', description: '默认短格式', priority: 90, matchPart: '', restPart: 'short' },
    { text: 'short-iso', type: 'subcommand', description: 'ISO 时间戳', priority: 88, matchPart: '', restPart: 'short-iso' },
    { text: 'short-precise', type: 'subcommand', description: '更高精度', priority: 85, matchPart: '', restPart: 'short-precise' },
    { text: 'cat', type: 'subcommand', description: '仅消息', priority: 80, matchPart: '', restPart: 'cat' },
    { text: 'json', type: 'subcommand', description: 'JSON', priority: 75, matchPart: '', restPart: 'json' },
    { text: 'json-pretty', type: 'subcommand', description: '漂亮 JSON', priority: 74, matchPart: '', restPart: 'json-pretty' },
    { text: 'verbose', type: 'subcommand', description: '详细字段', priority: 70, matchPart: '', restPart: 'verbose' },
];

const PRIORITY_HINTS: CompletionItem[] = [
    { text: '0', type: 'subcommand', description: 'emerg', priority: 90, matchPart: '', restPart: '0' },
    { text: '1', type: 'subcommand', description: 'alert', priority: 89, matchPart: '', restPart: '1' },
    { text: '2', type: 'subcommand', description: 'crit', priority: 88, matchPart: '', restPart: '2' },
    { text: '3', type: 'subcommand', description: 'err', priority: 87, matchPart: '', restPart: '3' },
    { text: '4', type: 'subcommand', description: 'warning', priority: 86, matchPart: '', restPart: '4' },
    { text: '5', type: 'subcommand', description: 'notice', priority: 85, matchPart: '', restPart: '5' },
    { text: '6', type: 'subcommand', description: 'info', priority: 84, matchPart: '', restPart: '6' },
    { text: '7', type: 'subcommand', description: 'debug', priority: 83, matchPart: '', restPart: '7' },
    { text: 'emerg', type: 'subcommand', description: '0', priority: 82, matchPart: '', restPart: 'emerg' },
    { text: 'alert', type: 'subcommand', description: '1', priority: 81, matchPart: '', restPart: 'alert' },
    { text: 'crit', type: 'subcommand', description: '2', priority: 80, matchPart: '', restPart: 'crit' },
    { text: 'err', type: 'subcommand', description: '3', priority: 79, matchPart: '', restPart: 'err' },
    { text: 'warning', type: 'subcommand', description: '4', priority: 78, matchPart: '', restPart: 'warning' },
    { text: 'notice', type: 'subcommand', description: '5', priority: 77, matchPart: '', restPart: 'notice' },
    { text: 'info', type: 'subcommand', description: '6', priority: 76, matchPart: '', restPart: 'info' },
    { text: 'debug', type: 'subcommand', description: '7', priority: 75, matchPart: '', restPart: 'debug' },
];

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

async function executeRemote(ctx: CompletionContext, command: string, timeout = 2500): Promise<string | null> {
    if (!ctx.sessionId || !ctx.electronAPI) return null;
    try {
        const result = await ctx.electronAPI.ssh?.executeCommand?.(ctx.sessionId, command, timeout);
        if (result?.success && result.data) {
            return String(result.data).trim();
        }
    } catch {
        // ignore
    }
    return null;
}

let serviceCache: { value: CompletionItem[]; ts: number } | null = null;
async function getServices(ctx: CompletionContext): Promise<CompletionItem[]> {
    const now = Date.now();
    if (serviceCache && now - serviceCache.ts < 5000) return serviceCache.value;

    const out = await executeRemote(ctx, `systemctl list-units --type=service --no-pager --plain 2>/dev/null | head -60 | awk '{print $1}'`);
    if (!out) return [];

    const items = out.split('\n')
        .map((line) => line.trim())
        .filter((line) => line && line.endsWith('.service'))
        .map((service) => service.replace('.service', ''));

    const value = items.map((name) => ({
        text: name,
        type: 'subcommand' as const,
        description: 'service',
        priority: 85,
        matchPart: '',
        restPart: name,
    }));
    serviceCache = { value, ts: now };
    return value;
}

const journalctlCommand: CommandDefinition = {
    name: 'journalctl',
    description: 'systemd 日志查看',
    options: [
        { text: '-u', type: 'option', description: '指定 unit', priority: 100, usage: '-u nginx' },
        { text: '--unit', type: 'option', description: '指定 unit', priority: 100 },
        { text: '-f', type: 'option', description: '实时跟踪', priority: 95 },
        { text: '-n', type: 'option', description: '最后 N 行', priority: 90, usage: '-n 200' },
        { text: '-b', type: 'option', description: '指定 boot', priority: 85, usage: '-b -1' },
        { text: '--since', type: 'option', description: '起始时间', priority: 82, usage: '--since \"1 hour ago\"' },
        { text: '--until', type: 'option', description: '结束时间', priority: 81, usage: '--until \"2026-03-11 10:00:00\"' },
        { text: '-p', type: 'option', description: '优先级', priority: 80, usage: '-p warning' },
        { text: '-o', type: 'option', description: '输出格式', priority: 75, usage: '-o short-iso' },
        { text: '--no-pager', type: 'option', description: '不分页', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-u' || prev === '--unit') {
            const items = await getServices(ctx);
            return filterHints(items, ctx.currentArg);
        }
        if (prev === '-o') {
            return filterHints(OUTPUT_FORMAT_HINTS, ctx.currentArg);
        }
        if (prev === '-p') {
            return filterHints(PRIORITY_HINTS, ctx.currentArg);
        }
        return [];
    }
};

export default journalctlCommand;


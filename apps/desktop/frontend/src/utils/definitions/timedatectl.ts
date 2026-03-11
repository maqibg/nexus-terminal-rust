
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * timedatectl 命令定义
 */

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

let tzCache: { value: CompletionItem[]; ts: number } | null = null;
async function getTimezones(ctx: CompletionContext): Promise<CompletionItem[]> {
    const now = Date.now();
    if (tzCache && now - tzCache.ts < 10000) return tzCache.value;

    const out = await executeRemote(ctx, `timedatectl list-timezones 2>/dev/null | head -200`);
    if (!out) return [];
    const items = out.split('\n').map((v) => v.trim()).filter(Boolean);
    const value = items.map((zone) => ({
        text: zone,
        type: 'subcommand' as const,
        description: 'timezone',
        priority: 80,
        matchPart: '',
        restPart: zone,
    }));
    tzCache = { value, ts: now };
    return value;
}

const timedatectlCommand: CommandDefinition = {
    name: 'timedatectl',
    description: '时间与时区管理',
    options: [
        { text: 'status', type: 'subcommand', description: '查看状态', priority: 100 },
        { text: 'set-time', type: 'subcommand', description: '设置时间', priority: 95, usage: `timedatectl set-time "2026-03-11 10:00:00"` },
        { text: 'set-timezone', type: 'subcommand', description: '设置时区', priority: 95, usage: 'timedatectl set-timezone Asia/Shanghai' },
        { text: 'list-timezones', type: 'subcommand', description: '列出时区', priority: 90 },
        { text: 'set-ntp', type: 'subcommand', description: '设置 NTP', priority: 85, usage: 'timedatectl set-ntp true' },
        { text: '--no-pager', type: 'option', description: '不分页', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        status: { name: 'status', description: '查看状态', options: [] },
        'set-time': { name: 'set-time', description: '设置时间', options: [] },
        'set-timezone': { name: 'set-timezone', description: '设置时区', options: [] },
        'list-timezones': { name: 'list-timezones', description: '列出时区', options: [] },
        'set-ntp': { name: 'set-ntp', description: '设置 NTP', options: [] },
    },
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const sub = ctx.args[1] ?? '';
        if (sub === 'set-timezone' && ctx.currentArgIndex >= 2) {
            const zones = await getTimezones(ctx);
            return filterHints(zones, ctx.currentArg);
        }
        if (sub === 'set-ntp' && ctx.currentArgIndex >= 2) {
            return filterHints(
                [
                    { text: 'true', type: 'subcommand', description: '启用 NTP', priority: 90, matchPart: '', restPart: 'true' },
                    { text: 'false', type: 'subcommand', description: '禁用 NTP', priority: 85, matchPart: '', restPart: 'false' },
                ],
                ctx.currentArg
            );
        }
        return [];
    }
};

export default timedatectlCommand;

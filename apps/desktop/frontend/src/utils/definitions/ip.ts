
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * ip 命令定义（iproute2）
 */

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

async function executeRemote(ctx: CompletionContext, command: string, timeout = 2000): Promise<string | null> {
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

let ifaceCache: { value: CompletionItem[]; ts: number } | null = null;
async function getInterfaces(ctx: CompletionContext): Promise<CompletionItem[]> {
    const now = Date.now();
    if (ifaceCache && now - ifaceCache.ts < 5000) return ifaceCache.value;

    const out = await executeRemote(ctx, `ip -o link show 2>/dev/null | awk -F': ' '{print $2}' | awk '{print $1}' | head -40`);
    if (!out) return [];
    const items = out.split('\n').map((v) => v.trim()).filter(Boolean);
    const value = items.map((name) => ({
        text: name,
        type: 'subcommand' as const,
        description: 'iface',
        priority: 80,
        matchPart: '',
        restPart: name,
    }));
    ifaceCache = { value, ts: now };
    return value;
}

const ipCommand: CommandDefinition = {
    name: 'ip',
    description: '网络配置工具',
    options: [
        { text: '-4', type: 'option', description: 'IPv4', priority: 95 },
        { text: '-6', type: 'option', description: 'IPv6', priority: 95 },
        { text: '-br', type: 'option', description: '简洁输出', priority: 90 },
        { text: '-s', type: 'option', description: '统计信息', priority: 85 },
        { text: '-d', type: 'option', description: '详细信息', priority: 80 },
        { text: '-json', type: 'option', description: 'JSON 输出', priority: 75 },
        { text: 'addr', type: 'subcommand', description: '地址', priority: 100 },
        { text: 'link', type: 'subcommand', description: '链路', priority: 95 },
        { text: 'route', type: 'subcommand', description: '路由', priority: 90 },
        { text: 'neigh', type: 'subcommand', description: '邻居', priority: 85 },
        { text: 'a', type: 'subcommand', description: 'addr (简写)', priority: 80 },
        { text: 'l', type: 'subcommand', description: 'link (简写)', priority: 80 },
        { text: 'r', type: 'subcommand', description: 'route (简写)', priority: 80 },
        { text: 'n', type: 'subcommand', description: 'neigh (简写)', priority: 80 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === 'dev') {
            const ifaces = await getInterfaces(ctx);
            return filterHints(ifaces, ctx.currentArg);
        }

        // ip link set <iface> ...
        const obj = ctx.args[1] ?? '';
        if ((obj === 'link' || obj === 'l') && ctx.args[2] === 'set' && ctx.currentArgIndex >= 3 && !ctx.currentArg.startsWith('-')) {
            const ifaces = await getInterfaces(ctx);
            return filterHints(ifaces, ctx.currentArg);
        }
        return [];
    }
};

export default ipCommand;


import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * jq 命令定义（常用参数）
 */
const jqCommand: CommandDefinition = {
    name: 'jq',
    description: 'JSON 处理与过滤',
    options: [
        { text: '-r', type: 'option', description: 'raw 输出（去引号）', priority: 100 },
        { text: '--raw-output', type: 'option', description: 'raw 输出（去引号）', priority: 95 },
        { text: '-c', type: 'option', description: '紧凑输出', priority: 92 },
        { text: '--compact-output', type: 'option', description: '紧凑输出', priority: 92 },
        { text: '-s', type: 'option', description: 'slurp 模式', priority: 88 },
        { text: '--slurp', type: 'option', description: 'slurp 模式', priority: 88 },
        { text: '-n', type: 'option', description: '不读取输入', priority: 85 },
        { text: '--null-input', type: 'option', description: '不读取输入', priority: 85 },
        { text: '-e', type: 'option', description: '设置退出码', priority: 80 },
        { text: '-S', type: 'option', description: 'key 排序', priority: 78 },
        { text: '--sort-keys', type: 'option', description: 'key 排序', priority: 78 },
        { text: '-M', type: 'option', description: '禁用颜色', priority: 75 },
        { text: '--monochrome-output', type: 'option', description: '禁用颜色', priority: 75 },
        { text: '-f', type: 'option', description: '从文件读取过滤器', priority: 72, usage: 'jq -f filter.jq file.json' },
        { text: '--from-file', type: 'option', description: '从文件读取过滤器', priority: 72 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if ((prev === '-f' || prev === '--from-file') && ctx.sessionId && ctx.electronAPI) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    },
};

export default jqCommand;

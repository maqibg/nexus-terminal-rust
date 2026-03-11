
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * npx 命令定义（等价于 npm exec 的常见用法）
 */

const NPX_TOOL_HINTS: CompletionItem[] = [
    { text: 'create-vite', type: 'hint', description: '创建 Vite 项目', priority: 95, usage: 'npx create-vite@latest' },
    { text: 'create-next-app', type: 'hint', description: '创建 Next.js 项目', priority: 90, usage: 'npx create-next-app@latest' },
    { text: 'create-react-app', type: 'hint', description: '创建 React 项目', priority: 88 },
    { text: 'eslint', type: 'hint', description: 'ESLint', priority: 80 },
    { text: 'prettier', type: 'hint', description: 'Prettier', priority: 78 },
    { text: 'ts-node', type: 'hint', description: 'TypeScript 运行时', priority: 76 },
    { text: 'tsc', type: 'hint', description: 'TypeScript 编译器', priority: 75 },
];

function filterStartsWith(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

const npxCommand: CommandDefinition = {
    name: 'npx',
    description: '执行 npm 包命令（npm exec）',
    options: [
        { text: '-y', type: 'option', description: '自动确认（yes）', priority: 90 },
        { text: '--yes', type: 'option', description: '自动确认（yes）', priority: 90 },
        { text: '--no', type: 'option', description: '拒绝自动安装', priority: 85 },
        { text: '-p', type: 'option', description: '指定包（package）', priority: 80, usage: 'npx -p pkg cmd' },
        { text: '--package', type: 'option', description: '指定包（package）', priority: 80 },
        { text: '-c', type: 'option', description: '执行 shell 命令（call）', priority: 70, usage: 'npx -c \"eslint .\"' },
        { text: '--call', type: 'option', description: '执行 shell 命令（call）', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-p' || prevArg === '--package') {
            return filterStartsWith(NPX_TOOL_HINTS, ctx.currentArg).map((item) => ({
                ...item,
                matchPart: ctx.currentArg,
                restPart: item.text.substring(ctx.currentArg.length),
            }));
        }
        if (!ctx.currentArg.startsWith('-')) {
            return filterStartsWith(NPX_TOOL_HINTS, ctx.currentArg).map((item) => ({
                ...item,
                matchPart: ctx.currentArg,
                restPart: item.text.substring(ctx.currentArg.length),
            }));
        }
        return [];
    },
};

export default npxCommand;


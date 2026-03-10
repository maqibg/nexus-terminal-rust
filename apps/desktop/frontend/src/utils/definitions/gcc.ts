
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteDirectories, getRemoteFiles } from '../providers/file-system';

/**
 * gcc 命令定义
 */

const gccCommand: CommandDefinition = {
    name: 'gcc',
    description: 'GNU C 编译器',
    options: [
        { text: '-o', type: 'option', description: '输出文件', priority: 100, usage: '-o app' },
        { text: '-c', type: 'option', description: '只编译不链接', priority: 95 },
        { text: '-Wall', type: 'option', description: '常用警告', priority: 90 },
        { text: '-Wextra', type: 'option', description: '更多警告', priority: 88 },
        { text: '-Werror', type: 'option', description: '警告当错误', priority: 86 },
        { text: '-g', type: 'option', description: '生成调试信息', priority: 84 },
        { text: '-O0', type: 'option', description: '不优化', priority: 82 },
        { text: '-O1', type: 'option', description: '优化等级 1', priority: 81 },
        { text: '-O2', type: 'option', description: '优化等级 2', priority: 80 },
        { text: '-O3', type: 'option', description: '优化等级 3', priority: 79 },
        { text: '-Os', type: 'option', description: '优化体积', priority: 78 },
        { text: '-std=c11', type: 'option', description: 'C11 标准', priority: 72 },
        { text: '-std=c17', type: 'option', description: 'C17 标准', priority: 71 },
        { text: '-std=gnu11', type: 'option', description: 'GNU C11', priority: 70 },
        { text: '-I', type: 'option', description: '头文件目录', priority: 95, usage: '-I include/', repeatable: true },
        { text: '-L', type: 'option', description: '库目录', priority: 90, usage: '-L /usr/local/lib', repeatable: true },
        { text: '-l', type: 'option', description: '链接库', priority: 88, usage: '-l ssl', repeatable: true },
        { text: '-D', type: 'option', description: '宏定义', priority: 86, usage: '-DDEBUG=1', repeatable: true },
        { text: '-U', type: 'option', description: '取消宏定义', priority: 84, usage: '-UDEBUG', repeatable: true },
        { text: '-pthread', type: 'option', description: '线程支持', priority: 80 },
        { text: '-shared', type: 'option', description: '生成共享库', priority: 75 },
        { text: '-fPIC', type: 'option', description: '位置无关代码', priority: 74 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-I' || prevArg === '-L') {
            return getRemoteDirectories(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }

        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    },
};

export default gccCommand;



import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * javac 命令定义
 */

const javacCommand: CommandDefinition = {
    name: 'javac',
    description: 'Java 编译器',
    options: [
        { text: '-d', type: 'option', description: '输出目录', priority: 100, usage: 'javac -d out src/Main.java' },
        { text: '-cp', type: 'option', description: '类路径', priority: 95, usage: 'javac -cp lib/* Main.java' },
        { text: '-classpath', type: 'option', description: '类路径', priority: 95 },
        { text: '--release', type: 'option', description: '目标 Java 版本', priority: 90, usage: '--release 17' },
        { text: '-source', type: 'option', description: '源版本', priority: 85, usage: '-source 17' },
        { text: '-target', type: 'option', description: '目标版本', priority: 80, usage: '-target 17' },
        { text: '-encoding', type: 'option', description: '源码编码', priority: 75, usage: '-encoding UTF-8' },
        { text: '-g', type: 'option', description: '生成调试信息', priority: 70 },
        { text: '-Werror', type: 'option', description: '警告视为错误', priority: 65 },
        { text: '-version', type: 'option', description: '显示版本', priority: 60 },
        { text: '-help', type: 'option', description: '显示帮助', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 55 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-d' || prevArg === '-cp' || prevArg === '-classpath') {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }

        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    },
};

export default javacCommand;


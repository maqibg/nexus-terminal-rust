
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * Java 命令定义
 */

const javaCommand: CommandDefinition = {
    name: 'java',
    description: 'Java 运行时',
    options: [
        { text: '-jar', type: 'option', description: '运行 jar 包', priority: 100, usage: 'java -jar app.jar' },
        { text: '-cp', type: 'option', description: '类路径', priority: 95, usage: 'java -cp .:lib/* Main' },
        { text: '-classpath', type: 'option', description: '类路径', priority: 95 },
        { text: '-D', type: 'option', description: '系统属性', priority: 90, usage: '-Dkey=value', repeatable: true },
        { text: '-Xms', type: 'option', description: '最小堆内存', priority: 80, usage: '-Xms256m' },
        { text: '-Xmx', type: 'option', description: '最大堆内存', priority: 80, usage: '-Xmx1g' },
        { text: '-ea', type: 'option', description: '启用断言', priority: 70 },
        { text: '-da', type: 'option', description: '禁用断言', priority: 65 },
        { text: '-version', type: 'option', description: '显示版本', priority: 60 },
        { text: '--version', type: 'option', description: '显示版本', priority: 60 },
        { text: '-help', type: 'option', description: '显示帮助', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 55 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-jar') {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        if (prevArg === '-cp' || prevArg === '-classpath') {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-')) {
            // 默认补全文件，常见为 jar 或 classpath
            return getRemoteFiles(ctx.sessionId, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    },
};

export default javaCommand;


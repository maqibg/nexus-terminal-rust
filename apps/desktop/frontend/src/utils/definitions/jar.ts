
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * jar 命令定义
 */

const jarCommand: CommandDefinition = {
    name: 'jar',
    description: 'JAR 打包/解包工具',
    options: [
        { text: '-c', type: 'option', description: '创建 jar（create）', priority: 100, usage: 'jar -cf app.jar -C out .' },
        { text: '--create', type: 'option', description: '创建 jar', priority: 100 },
        { text: '-u', type: 'option', description: '更新 jar（update）', priority: 95, usage: 'jar -uf app.jar -C out .' },
        { text: '--update', type: 'option', description: '更新 jar', priority: 95 },
        { text: '-t', type: 'option', description: '列出 jar 内容（list）', priority: 90, usage: 'jar -tf app.jar' },
        { text: '--list', type: 'option', description: '列出 jar 内容', priority: 90 },
        { text: '-x', type: 'option', description: '解压 jar（extract）', priority: 85, usage: 'jar -xf app.jar' },
        { text: '--extract', type: 'option', description: '解压 jar', priority: 85 },
        { text: '-f', type: 'option', description: '指定 jar 文件（file）', priority: 80, usage: 'jar -cf app.jar ...' },
        { text: '--file', type: 'option', description: '指定 jar 文件', priority: 80 },
        { text: '-m', type: 'option', description: '指定 manifest（manifest）', priority: 75, usage: 'jar -cfm app.jar MANIFEST.MF -C out .' },
        { text: '--manifest', type: 'option', description: '指定 manifest', priority: 75 },
        { text: '-v', type: 'option', description: '详细输出（verbose）', priority: 70 },
        { text: '--verbose', type: 'option', description: '详细输出', priority: 70 },
        { text: '-0', type: 'option', description: '不压缩（store only）', priority: 65 },
        { text: '--no-compress', type: 'option', description: '不压缩', priority: 65 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-f' || prevArg === '--file' || prevArg === '-m' || prevArg === '--manifest') {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    },
};

export default jarCommand;


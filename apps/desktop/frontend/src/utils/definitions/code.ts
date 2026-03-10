
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * code 命令 - VS Code CLI
 */

const codeCommand: CommandDefinition = {
    name: 'code',
    description: 'VS Code 命令行工具',
    options: [
        { text: '-n', type: 'option', description: '新窗口打开', priority: 95 },
        { text: '--new-window', type: 'option', description: '新窗口打开', priority: 95 },
        { text: '-r', type: 'option', description: '复用窗口打开', priority: 90 },
        { text: '--reuse-window', type: 'option', description: '复用窗口打开', priority: 90 },
        { text: '-g', type: 'option', description: '跳转到文件:行:列', priority: 85, usage: 'code -g file:10:2' },
        { text: '--goto', type: 'option', description: '跳转到文件:行:列', priority: 85 },
        { text: '--diff', type: 'option', description: '打开 diff', priority: 80, usage: 'code --diff a b' },
        { text: '--wait', type: 'option', description: '等待关闭后退出', priority: 75 },
        { text: '--install-extension', type: 'option', description: '安装扩展', priority: 70, usage: '--install-extension ms-python.python' },
        { text: '--uninstall-extension', type: 'option', description: '卸载扩展', priority: 65, usage: '--uninstall-extension ms-python.python' },
        { text: '--list-extensions', type: 'option', description: '列出扩展', priority: 60 },
        { text: '--disable-extensions', type: 'option', description: '禁用扩展', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-g' || prevArg === '--goto') {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg, ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    },
};

export default codeCommand;



import type { CommandDefinition, CompletionContext, CompletionItem } from '../../types';
import { suggestRemotePaths } from './helpers';

const uvToolInstall: CommandDefinition = {
    name: 'install',
    description: '安装工具（Python 包提供的命令）',
    options: [
        { text: '--with', type: 'option', description: '附加依赖', priority: 90, usage: '--with ruff' },
        { text: '--with-requirements', type: 'option', description: '从 requirements 引入附加依赖', priority: 85 },
        { text: '-e', type: 'option', description: '可编辑安装目标包', priority: 80 },
        { text: '--editable', type: 'option', description: '可编辑安装目标包', priority: 80 },
        { text: '--force', type: 'option', description: '强制安装', priority: 75 },
        { text: '-p', type: 'option', description: '指定 Python 解释器', priority: 70, usage: '--python python3.12' },
        { text: '--python', type: 'option', description: '指定 Python 解释器', priority: 70 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['--with-requirements', '-p', '--python'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

export const uvTool: CommandDefinition = {
    name: 'tool',
    description: '安装/运行 Python 工具',
    options: [
        { text: 'run', type: 'subcommand', description: '运行工具', priority: 100, usage: 'uv tool run ruff --version' },
        { text: 'install', type: 'subcommand', description: '安装工具', priority: 95, usage: 'uv tool install ruff' },
        { text: 'upgrade', type: 'subcommand', description: '升级已安装工具', priority: 90 },
        { text: 'list', type: 'subcommand', description: '列出工具', priority: 85 },
        { text: 'uninstall', type: 'subcommand', description: '卸载工具', priority: 80 },
        { text: 'update-shell', type: 'subcommand', description: '确保 tools 目录在 PATH', priority: 75 },
        { text: 'dir', type: 'subcommand', description: '显示 tools 目录', priority: 70 },
    ],
    subcommands: {
        run: { name: 'run', description: '运行工具', options: [] },
        install: uvToolInstall,
        upgrade: { name: 'upgrade', description: '升级已安装工具', options: [] },
        list: { name: 'list', description: '列出工具', options: [] },
        uninstall: { name: 'uninstall', description: '卸载工具', options: [] },
        'update-shell': { name: 'update-shell', description: '更新 shell PATH', options: [] },
        dir: { name: 'dir', description: '显示 tools 目录', options: [] },
    },
};


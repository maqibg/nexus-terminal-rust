
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * pip 命令定义
 */
const pipInstall: CommandDefinition = {
    name: 'install',
    description: '安装包',
    options: [
        { text: '-U', type: 'option', description: '升级安装', priority: 100 },
        { text: '--upgrade', type: 'option', description: '升级安装', priority: 100 },
        { text: '-r', type: 'option', description: 'requirements 文件', priority: 95, usage: '-r requirements.txt' },
        { text: '--requirement', type: 'option', description: 'requirements 文件', priority: 95 },
        { text: '--no-deps', type: 'option', description: '不安装依赖', priority: 80 },
        { text: '--user', type: 'option', description: '安装到用户目录', priority: 75 },
        { text: '--proxy', type: 'option', description: '代理', priority: 70, usage: '--proxy http://127.0.0.1:7890' },
        { text: '--timeout', type: 'option', description: '超时(秒)', priority: 65, usage: '--timeout 15' },
        { text: '-i', type: 'option', description: 'index-url', priority: 60, usage: '-i https://pypi.org/simple' },
        { text: '--index-url', type: 'option', description: 'index-url', priority: 60 },
        { text: '--extra-index-url', type: 'option', description: '额外 index', priority: 55 },
        { text: '--trusted-host', type: 'option', description: '信任 host', priority: 50 },
        { text: '--no-cache-dir', type: 'option', description: '禁用缓存', priority: 45 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-r' || prev === '--requirement') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    }
};

const pipUninstall: CommandDefinition = {
    name: 'uninstall',
    description: '卸载包',
    options: [
        { text: '-y', type: 'option', description: '自动确认', priority: 100 },
        { text: '--yes', type: 'option', description: '自动确认', priority: 100 },
    ],
};

const pipCommand: CommandDefinition = {
    name: 'pip',
    description: 'Python 包管理器',
    options: [
        { text: 'install', type: 'subcommand', description: '安装包', priority: 100 },
        { text: 'uninstall', type: 'subcommand', description: '卸载包', priority: 95 },
        { text: 'list', type: 'subcommand', description: '列出已安装包', priority: 90 },
        { text: 'show', type: 'subcommand', description: '显示包信息', priority: 85 },
        { text: 'freeze', type: 'subcommand', description: '导出依赖', priority: 80 },
        { text: 'check', type: 'subcommand', description: '检查依赖冲突', priority: 75 },
        { text: 'download', type: 'subcommand', description: '下载包', priority: 70 },
        { text: 'cache', type: 'subcommand', description: '缓存管理', priority: 65 },
        { text: '--version', type: 'option', description: '显示版本', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        install: pipInstall,
        uninstall: pipUninstall,
    },
};

export default pipCommand;


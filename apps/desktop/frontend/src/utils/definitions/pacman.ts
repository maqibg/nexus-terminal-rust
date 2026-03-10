
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * pacman 命令定义（Arch）
 */
const pacmanCommand: CommandDefinition = {
    name: 'pacman',
    description: 'Arch 包管理器',
    options: [
        { text: '-S', type: 'subcommand', description: '安装包', priority: 100, usage: 'pacman -S pkg' },
        { text: '-R', type: 'subcommand', description: '删除包', priority: 95, usage: 'pacman -R pkg' },
        { text: '-Q', type: 'subcommand', description: '查询已安装包', priority: 90 },
        { text: '-Ss', type: 'subcommand', description: '搜索包', priority: 85, usage: 'pacman -Ss keyword' },
        { text: '-Syu', type: 'subcommand', description: '更新系统', priority: 80, usage: 'pacman -Syu' },
        { text: '-U', type: 'option', description: '安装本地包文件', priority: 78, usage: 'pacman -U pkg.pkg.tar.zst' },
        { text: '--noconfirm', type: 'option', description: '不询问确认', priority: 75 },
        { text: '--needed', type: 'option', description: '跳过已安装', priority: 70 },
        { text: '--overwrite', type: 'option', description: '覆盖文件', priority: 65, usage: '--overwrite "*"' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-U') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    }
};

export default pacmanCommand;


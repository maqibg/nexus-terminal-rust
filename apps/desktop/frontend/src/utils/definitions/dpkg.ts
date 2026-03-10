
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * dpkg 命令定义
 */
const dpkgCommand: CommandDefinition = {
    name: 'dpkg',
    description: 'Debian 包管理工具',
    options: [
        { text: '-i', type: 'option', description: '安装 .deb', priority: 100, usage: 'dpkg -i pkg.deb' },
        { text: '-r', type: 'option', description: '删除包', priority: 90, usage: 'dpkg -r pkg' },
        { text: '-P', type: 'option', description: '彻底删除(含配置)', priority: 85, usage: 'dpkg -P pkg' },
        { text: '-l', type: 'option', description: '列出已安装包', priority: 80, usage: 'dpkg -l | grep xxx' },
        { text: '-L', type: 'option', description: '列出包文件', priority: 75, usage: 'dpkg -L pkg' },
        { text: '-S', type: 'option', description: '反查文件属于哪个包', priority: 70, usage: 'dpkg -S /path/file' },
        { text: '-s', type: 'option', description: '包状态', priority: 65, usage: 'dpkg -s pkg' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prev === '-i') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        if (prev === '-S') {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    }
};

export default dpkgCommand;



import type { CommandDefinition } from '../types';

/**
 * apt-cache 命令定义（Debian/Ubuntu）
 */

const aptCacheCommand: CommandDefinition = {
    name: 'apt-cache',
    description: '查询 APT 缓存/包信息',
    options: [
        { text: 'search', type: 'subcommand', description: '搜索包', priority: 100, usage: 'apt-cache search nginx' },
        { text: 'show', type: 'subcommand', description: '显示包信息', priority: 95, usage: 'apt-cache show nginx' },
        { text: 'policy', type: 'subcommand', description: '查看候选版本/来源', priority: 90, usage: 'apt-cache policy nginx' },
        { text: 'depends', type: 'subcommand', description: '查看依赖', priority: 85, usage: 'apt-cache depends nginx' },
        { text: 'rdepends', type: 'subcommand', description: '反向依赖', priority: 80, usage: 'apt-cache rdepends nginx' },
        { text: 'pkgnames', type: 'subcommand', description: '列出包名', priority: 75, usage: 'apt-cache pkgnames | head' },
        { text: 'stats', type: 'subcommand', description: '缓存统计', priority: 70, usage: 'apt-cache stats' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        search: { name: 'search', description: '搜索包', options: [] },
        show: { name: 'show', description: '显示包信息', options: [] },
        policy: { name: 'policy', description: '查看候选版本/来源', options: [] },
        depends: { name: 'depends', description: '查看依赖', options: [] },
        rdepends: { name: 'rdepends', description: '反向依赖', options: [] },
        pkgnames: { name: 'pkgnames', description: '列出包名', options: [] },
        stats: { name: 'stats', description: '缓存统计', options: [] },
    },
};

export default aptCacheCommand;


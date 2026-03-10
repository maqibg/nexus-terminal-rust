
import type { CommandDefinition } from '../types';

/**
 * apt-get 命令定义（Debian/Ubuntu）
 */
const aptGetCommand: CommandDefinition = {
    name: 'apt-get',
    description: 'apt-get 包管理器',
    options: [
        { text: 'update', type: 'subcommand', description: '更新索引', priority: 100 },
        { text: 'upgrade', type: 'subcommand', description: '升级包', priority: 95 },
        { text: 'dist-upgrade', type: 'subcommand', description: '发行版升级', priority: 90 },
        { text: 'install', type: 'subcommand', description: '安装包', priority: 90 },
        { text: 'remove', type: 'subcommand', description: '删除包', priority: 85 },
        { text: 'purge', type: 'subcommand', description: '删除包(含配置)', priority: 80 },
        { text: 'autoremove', type: 'subcommand', description: '移除无用依赖', priority: 78 },
        { text: 'clean', type: 'subcommand', description: '清理缓存', priority: 70 },
        { text: '-y', type: 'option', description: '自动确认', priority: 95 },
        { text: '-qq', type: 'option', description: '更静默', priority: 80 },
        { text: '--no-install-recommends', type: 'option', description: '不装推荐包', priority: 75 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default aptGetCommand;


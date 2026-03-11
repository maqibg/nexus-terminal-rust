
import type { CommandDefinition } from '../types';

/**
 * apt 命令定义（Debian/Ubuntu）
 */
const aptCommand: CommandDefinition = {
    name: 'apt',
    description: 'Debian/Ubuntu 包管理器',
    options: [
        { text: 'update', type: 'subcommand', description: '更新索引', priority: 100 },
        { text: 'upgrade', type: 'subcommand', description: '升级包', priority: 95 },
        { text: 'full-upgrade', type: 'subcommand', description: '完整升级', priority: 90 },
        { text: 'install', type: 'subcommand', description: '安装包', priority: 90 },
        { text: 'remove', type: 'subcommand', description: '删除包', priority: 85 },
        { text: 'purge', type: 'subcommand', description: '删除包(含配置)', priority: 80 },
        { text: 'autoremove', type: 'subcommand', description: '移除无用依赖', priority: 78 },
        { text: 'search', type: 'subcommand', description: '搜索包', priority: 75 },
        { text: 'show', type: 'subcommand', description: '显示包信息', priority: 70 },
        { text: 'list', type: 'subcommand', description: '列出包', priority: 65 },
        { text: 'policy', type: 'subcommand', description: '策略/候选版本', priority: 60 },
        { text: '-y', type: 'option', description: '自动确认', priority: 95 },
        { text: '--yes', type: 'option', description: '自动确认', priority: 95 },
        { text: '-qq', type: 'option', description: '更静默', priority: 80 },
        { text: '--no-install-recommends', type: 'option', description: '不装推荐包', priority: 75 },
        { text: '--reinstall', type: 'option', description: '重新安装', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        update: { name: 'update', description: '更新索引', options: [] },
        upgrade: { name: 'upgrade', description: '升级包', options: [] },
        'full-upgrade': { name: 'full-upgrade', description: '完整升级', options: [] },
        install: { name: 'install', description: '安装包', options: [] },
        remove: { name: 'remove', description: '删除包', options: [] },
        purge: { name: 'purge', description: '删除包(含配置)', options: [] },
        autoremove: { name: 'autoremove', description: '移除无用依赖', options: [] },
        search: { name: 'search', description: '搜索包', options: [] },
        show: { name: 'show', description: '显示包信息', options: [] },
        list: { name: 'list', description: '列出包', options: [] },
        policy: { name: 'policy', description: '策略/候选版本', options: [] },
    },
};

export default aptCommand;

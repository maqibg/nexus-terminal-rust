
import type { CommandDefinition } from '../types';

/**
 * dnf 命令定义（Fedora/RHEL）
 */
const dnfCommand: CommandDefinition = {
    name: 'dnf',
    description: 'Fedora/RHEL 包管理器',
    options: [
        { text: 'install', type: 'subcommand', description: '安装包', priority: 100 },
        { text: 'remove', type: 'subcommand', description: '删除包', priority: 95 },
        { text: 'update', type: 'subcommand', description: '更新包', priority: 90 },
        { text: 'upgrade', type: 'subcommand', description: '升级包', priority: 90 },
        { text: 'search', type: 'subcommand', description: '搜索包', priority: 85 },
        { text: 'info', type: 'subcommand', description: '包信息', priority: 80 },
        { text: 'list', type: 'subcommand', description: '列出包', priority: 75 },
        { text: 'repolist', type: 'subcommand', description: '列出仓库', priority: 70 },
        { text: 'clean', type: 'subcommand', description: '清理缓存', priority: 65 },
        { text: 'makecache', type: 'subcommand', description: '生成缓存', priority: 60 },
        { text: '-y', type: 'option', description: '自动确认', priority: 95 },
        { text: '-q', type: 'option', description: '安静模式', priority: 80 },
        { text: '--disablerepo', type: 'option', description: '禁用仓库', priority: 75, usage: '--disablerepo=epel' },
        { text: '--enablerepo', type: 'option', description: '启用仓库', priority: 75, usage: '--enablerepo=epel' },
        { text: '--nogpgcheck', type: 'option', description: '跳过 GPG 校验', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        install: { name: 'install', description: '安装包', options: [] },
        remove: { name: 'remove', description: '删除包', options: [] },
        update: { name: 'update', description: '更新包', options: [] },
        upgrade: { name: 'upgrade', description: '升级包', options: [] },
        search: { name: 'search', description: '搜索包', options: [] },
        info: { name: 'info', description: '包信息', options: [] },
        list: { name: 'list', description: '列出包', options: [] },
        repolist: { name: 'repolist', description: '列出仓库', options: [] },
        clean: { name: 'clean', description: '清理缓存', options: [] },
        makecache: { name: 'makecache', description: '生成缓存', options: [] },
    },
};

export default dnfCommand;

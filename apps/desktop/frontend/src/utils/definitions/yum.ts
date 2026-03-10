
import type { CommandDefinition } from '../types';

/**
 * yum 命令定义（RHEL/CentOS）
 */
const yumCommand: CommandDefinition = {
    name: 'yum',
    description: 'RHEL/CentOS 包管理器',
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
        { text: '-y', type: 'option', description: '自动确认', priority: 95 },
        { text: '-q', type: 'option', description: '安静模式', priority: 80 },
        { text: '--disablerepo', type: 'option', description: '禁用仓库', priority: 75, usage: '--disablerepo=epel' },
        { text: '--enablerepo', type: 'option', description: '启用仓库', priority: 75, usage: '--enablerepo=epel' },
        { text: '--nogpgcheck', type: 'option', description: '跳过 GPG 校验', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default yumCommand;


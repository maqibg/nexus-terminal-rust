
import type { CommandDefinition } from '../../types';

export const uvPython: CommandDefinition = {
    name: 'python',
    description: '管理 Python 安装与版本',
    options: [
        { text: 'list', type: 'subcommand', description: '列出可用 Python', priority: 100 },
        { text: 'install', type: 'subcommand', description: '安装 Python', priority: 95 },
        { text: 'find', type: 'subcommand', description: '查找 Python', priority: 90 },
        { text: 'pin', type: 'subcommand', description: '固定 Python 版本', priority: 85 },
        { text: 'dir', type: 'subcommand', description: '显示安装目录', priority: 80 },
        { text: 'uninstall', type: 'subcommand', description: '卸载 Python', priority: 75 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出 Python 安装', options: [] },
        install: { name: 'install', description: '安装 Python', options: [] },
        find: { name: 'find', description: '查找 Python', options: [] },
        pin: { name: 'pin', description: '固定 Python 版本', options: [] },
        dir: { name: 'dir', description: '显示 uv Python 目录', options: [] },
        uninstall: { name: 'uninstall', description: '卸载 Python', options: [] },
    },
};


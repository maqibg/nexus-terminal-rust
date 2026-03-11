
import type { CommandDefinition } from '../../types';

export const uvCache: CommandDefinition = {
    name: 'cache',
    description: '管理 uv 缓存',
    options: [
        { text: 'clean', type: 'subcommand', description: '清理缓存', priority: 100 },
        { text: 'prune', type: 'subcommand', description: '清除不可达对象', priority: 90 },
        { text: 'dir', type: 'subcommand', description: '显示缓存目录', priority: 80 },
    ],
    subcommands: {
        clean: { name: 'clean', description: '清理缓存', options: [] },
        prune: { name: 'prune', description: '清理不可达对象', options: [] },
        dir: { name: 'dir', description: '显示缓存目录', options: [] },
    },
};


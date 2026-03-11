
import type { CommandDefinition } from '../../types';

export const uvSelf: CommandDefinition = {
    name: 'self',
    description: '管理 uv 可执行文件',
    options: [
        { text: 'update', type: 'subcommand', description: '更新 uv', priority: 100 },
        { text: 'version', type: 'subcommand', description: '显示 uv 版本', priority: 90 },
    ],
    subcommands: {
        update: { name: 'update', description: '更新 uv', options: [] },
        version: { name: 'version', description: '显示 uv 版本', options: [] },
    },
};


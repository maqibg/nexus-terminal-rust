import type { CommandDefinition } from '../types';

/**
 * time 命令定义
 */
const timeCommand: CommandDefinition = {
    name: 'time',
    description: '统计命令执行耗时',
    options: [
        { text: '-p', type: 'option', description: 'POSIX 格式输出', priority: 90 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
};

export default timeCommand;

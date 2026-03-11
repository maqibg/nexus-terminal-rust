import type { CommandDefinition } from '../types';

/**
 * nohup 命令定义
 */
const nohupCommand: CommandDefinition = {
    name: 'nohup',
    description: '忽略挂起信号并后台运行',
    options: [
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
};

export default nohupCommand;

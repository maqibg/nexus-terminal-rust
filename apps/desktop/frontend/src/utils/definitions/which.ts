
import type { CommandDefinition } from '../types';

/**
 * which 命令定义
 */

const whichCommand: CommandDefinition = {
    name: 'which',
    description: '查找可执行文件路径',
    options: [
        { text: '-a', type: 'option', description: '显示所有匹配项', priority: 90, usage: 'which -a python' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 48 },
    ],
};

export default whichCommand;


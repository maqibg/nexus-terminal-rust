
import type { CommandDefinition } from '../types';

/**
 * uname 命令定义
 */
const unameCommand: CommandDefinition = {
    name: 'uname',
    description: '显示系统信息',
    options: [
        { text: '-a', type: 'option', description: '全部信息', priority: 100, usage: 'uname -a' },
        { text: '-s', type: 'option', description: '内核名称', priority: 80 },
        { text: '-n', type: 'option', description: '主机名', priority: 80 },
        { text: '-r', type: 'option', description: '内核版本', priority: 80 },
        { text: '-v', type: 'option', description: '内核发布信息', priority: 75 },
        { text: '-m', type: 'option', description: '机器硬件名', priority: 75 },
        { text: '-p', type: 'option', description: '处理器类型', priority: 70 },
        { text: '-i', type: 'option', description: '硬件平台', priority: 70 },
        { text: '-o', type: 'option', description: '操作系统', priority: 65 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
};

export default unameCommand;


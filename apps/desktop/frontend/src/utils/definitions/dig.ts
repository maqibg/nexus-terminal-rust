
import type { CommandDefinition } from '../types';

/**
 * dig 命令定义
 */
const digCommand: CommandDefinition = {
    name: 'dig',
    description: 'DNS 查询工具',
    options: [
        { text: '+short', type: 'option', description: '简短输出', priority: 95 },
        { text: '+trace', type: 'option', description: '递归跟踪', priority: 90 },
        { text: '+nocmd', type: 'option', description: '不显示命令', priority: 80 },
        { text: '+noall', type: 'option', description: '不显示默认段', priority: 75 },
        { text: '+answer', type: 'option', description: '只显示 answer', priority: 70 },
        { text: '-x', type: 'option', description: '反向解析', priority: 65, usage: '-x 8.8.8.8' },
        { text: '-t', type: 'option', description: '指定类型', priority: 60, usage: '-t A' },
        { text: '@8.8.8.8', type: 'hint', description: '指定 DNS 服务器', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default digCommand;


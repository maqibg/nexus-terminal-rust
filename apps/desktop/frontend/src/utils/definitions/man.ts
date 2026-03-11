
import type { CommandDefinition } from '../types';

/**
 * man 命令定义
 */

const manCommand: CommandDefinition = {
    name: 'man',
    description: '查看命令/系统文档',
    options: [
        { text: '-k', type: 'option', description: '按关键字搜索（apropos）', priority: 90, usage: 'man -k network' },
        { text: '-f', type: 'option', description: '显示简短描述（whatis）', priority: 85, usage: 'man -f ls' },
        { text: '-a', type: 'option', description: '显示所有匹配页面', priority: 80 },
        { text: '-P', type: 'option', description: '指定 pager', priority: 75, usage: 'man -P less ls' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default manCommand;



import type { CommandDefinition } from '../types';

/**
 * xargs 命令定义
 */
const xargsCommand: CommandDefinition = {
    name: 'xargs',
    description: '把输入转成命令参数',
    options: [
        { text: '-0', type: 'option', description: 'NUL 分隔输入', priority: 95 },
        { text: '-n', type: 'option', description: '每次最多 N 个参数', priority: 90, usage: '-n 10' },
        { text: '-P', type: 'option', description: '并行数', priority: 85, usage: '-P 4' },
        { text: '-I', type: 'option', description: '替换字符串', priority: 80, usage: `-I{} echo {}` },
        { text: '-t', type: 'option', description: '打印执行命令', priority: 70 },
        { text: '-r', type: 'option', description: '无输入不执行', priority: 65 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default xargsCommand;


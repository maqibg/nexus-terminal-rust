import type { CommandDefinition } from '../types';

/**
 * env 命令定义
 */
const envCommand: CommandDefinition = {
    name: 'env',
    description: '设置环境变量并执行命令',
    options: [
        { text: '-i', type: 'option', description: '忽略继承环境变量', priority: 100, usage: 'env -i VAR=1 cmd' },
        { text: '--ignore-environment', type: 'option', description: '忽略继承环境变量', priority: 95 },
        { text: '-u', type: 'option', description: '取消指定变量', priority: 90, usage: 'env -u PATH cmd' },
        { text: '--unset', type: 'option', description: '取消指定变量', priority: 90, usage: '--unset PATH' },
        { text: '-C', type: 'option', description: '切换目录执行', priority: 85, usage: 'env -C /tmp cmd' },
        { text: '--chdir', type: 'option', description: '切换目录执行', priority: 85, usage: '--chdir /tmp' },
        { text: '-0', type: 'option', description: '以 NUL 分隔输出', priority: 70 },
        { text: '--null', type: 'option', description: '以 NUL 分隔输出', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
};

export default envCommand;

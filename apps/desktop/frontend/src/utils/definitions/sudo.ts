import type { CommandDefinition } from '../types';

/**
 * sudo 命令定义
 */
const sudoCommand: CommandDefinition = {
    name: 'sudo',
    description: '以 root/其他用户执行命令',
    options: [
        { text: '-u', type: 'option', description: '指定用户', priority: 100, usage: 'sudo -u root cmd' },
        { text: '--user', type: 'option', description: '指定用户', priority: 95, usage: '--user=root' },
        { text: '-g', type: 'option', description: '指定组', priority: 90, usage: 'sudo -g wheel cmd' },
        { text: '--group', type: 'option', description: '指定组', priority: 90, usage: '--group=wheel' },
        { text: '-E', type: 'option', description: '保留环境变量', priority: 85 },
        { text: '-H', type: 'option', description: '设置 HOME', priority: 80 },
        { text: '-i', type: 'option', description: '登录 shell', priority: 80 },
        { text: '-s', type: 'option', description: '使用 shell 执行', priority: 75 },
        { text: '-k', type: 'option', description: '使缓存失效', priority: 70 },
        { text: '-n', type: 'option', description: '非交互模式', priority: 70 },
        { text: '-v', type: 'option', description: '刷新缓存', priority: 65 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
};

export default sudoCommand;

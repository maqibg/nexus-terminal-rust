import type { CommandDefinition } from '../types';

/**
 * screen 命令定义（常用参数）
 */
const screenCommand: CommandDefinition = {
    name: 'screen',
    description: '终端复用器 (GNU Screen)',
    options: [
        { text: '-ls', type: 'option', description: '列出会话', priority: 100, usage: 'screen -ls' },
        { text: '-r', type: 'option', description: '恢复会话', priority: 95, usage: 'screen -r <id>' },
        { text: '-R', type: 'option', description: '恢复/创建会话', priority: 92, usage: 'screen -R name' },
        { text: '-S', type: 'option', description: '指定会话名', priority: 90, usage: 'screen -S name' },
        { text: '-d', type: 'option', description: '分离会话', priority: 85 },
        { text: '-m', type: 'option', description: '强制新建会话', priority: 85 },
        { text: '-D', type: 'option', description: '强制分离并恢复', priority: 80 },
        { text: '-x', type: 'option', description: '多终端附加', priority: 75 },
        { text: '-p', type: 'option', description: '指定窗口', priority: 70, usage: 'screen -p 0 -X stuff' },
        { text: '-X', type: 'option', description: '发送命令', priority: 70, usage: 'screen -X stuff \"cmd\"' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
};

export default screenCommand;

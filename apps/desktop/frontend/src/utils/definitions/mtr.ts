
import type { CommandDefinition } from '../types';

/**
 * mtr 命令定义
 */
const mtrCommand: CommandDefinition = {
    name: 'mtr',
    description: '网络链路诊断',
    options: [
        { text: '-r', type: 'option', description: '报告模式', priority: 95 },
        { text: '-c', type: 'option', description: '发送次数', priority: 90, usage: '-c 100' },
        { text: '-n', type: 'option', description: '不解析域名', priority: 85 },
        { text: '-w', type: 'option', description: '宽输出', priority: 80 },
        { text: '-b', type: 'option', description: '显示 IP/域名', priority: 75 },
        { text: '-4', type: 'option', description: 'IPv4', priority: 70 },
        { text: '-6', type: 'option', description: 'IPv6', priority: 70 },
        { text: '-T', type: 'option', description: 'TCP 模式', priority: 65 },
        { text: '-u', type: 'option', description: 'UDP 模式', priority: 60 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default mtrCommand;


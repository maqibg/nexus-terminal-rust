
import type { CommandDefinition } from '../types';

/**
 * traceroute 命令定义
 */
const tracerouteCommand: CommandDefinition = {
    name: 'traceroute',
    description: '路由追踪',
    options: [
        { text: '-n', type: 'option', description: '不解析域名', priority: 90 },
        { text: '-I', type: 'option', description: 'ICMP 模式', priority: 85 },
        { text: '-T', type: 'option', description: 'TCP 模式', priority: 80 },
        { text: '-p', type: 'option', description: '端口', priority: 75, usage: '-p 443' },
        { text: '-m', type: 'option', description: '最大跳数', priority: 70, usage: '-m 30' },
        { text: '-q', type: 'option', description: '每跳探测次数', priority: 65, usage: '-q 3' },
        { text: '-w', type: 'option', description: '每跳超时(秒)', priority: 60, usage: '-w 2' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default tracerouteCommand;


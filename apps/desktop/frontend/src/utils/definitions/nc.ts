
import type { CommandDefinition } from '../types';

/**
 * nc (netcat) 命令定义
 */
const ncCommand: CommandDefinition = {
    name: 'nc',
    description: 'netcat 网络工具',
    options: [
        { text: '-l', type: 'option', description: '监听模式', priority: 95, usage: 'nc -l 8080' },
        { text: '-k', type: 'option', description: '保持监听', priority: 90 },
        { text: '-v', type: 'option', description: '详细输出', priority: 85 },
        { text: '-vv', type: 'option', description: '更详细输出', priority: 80 },
        { text: '-z', type: 'option', description: '扫描模式', priority: 75, usage: 'nc -zv host 1-1024' },
        { text: '-w', type: 'option', description: '超时(秒)', priority: 70, usage: '-w 3' },
        { text: '-p', type: 'option', description: '本地端口', priority: 65, usage: '-p 12345' },
        { text: '-u', type: 'option', description: 'UDP', priority: 60 },
        { text: '-n', type: 'option', description: '不解析域名', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default ncCommand;


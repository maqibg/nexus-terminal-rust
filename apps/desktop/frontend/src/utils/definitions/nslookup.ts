
import type { CommandDefinition } from '../types';

/**
 * nslookup 命令定义
 */
const nslookupCommand: CommandDefinition = {
    name: 'nslookup',
    description: 'DNS 查询工具',
    options: [
        { text: '-type=A', type: 'option', description: '查询 A 记录', priority: 85 },
        { text: '-type=AAAA', type: 'option', description: '查询 AAAA 记录', priority: 84 },
        { text: '-type=TXT', type: 'option', description: '查询 TXT 记录', priority: 83 },
        { text: '-type=MX', type: 'option', description: '查询 MX 记录', priority: 82 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default nslookupCommand;


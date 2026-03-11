import type { CommandDefinition } from '../types';

/**
 * rsync 命令定义（常用参数）
 */
const rsyncCommand: CommandDefinition = {
    name: 'rsync',
    description: '文件同步/复制工具',
    options: [
        { text: '-a', type: 'option', description: '归档模式', priority: 100, usage: 'rsync -a src/ dest/' },
        { text: '--archive', type: 'option', description: '归档模式', priority: 95 },
        { text: '-v', type: 'option', description: '详细输出', priority: 90 },
        { text: '--verbose', type: 'option', description: '详细输出', priority: 90 },
        { text: '-z', type: 'option', description: '压缩传输', priority: 85 },
        { text: '--compress', type: 'option', description: '压缩传输', priority: 85 },
        { text: '--delete', type: 'option', description: '删除目标多余文件', priority: 80 },
        { text: '--progress', type: 'option', description: '显示进度', priority: 75 },
        { text: '--dry-run', type: 'option', description: '演练，不实际修改', priority: 70, usage: 'rsync --dry-run -a src/ dest/' },
        { text: '--exclude', type: 'option', description: '排除模式', priority: 65, usage: '--exclude node_modules' },
        { text: '--exclude-from', type: 'option', description: '从文件读取排除规则', priority: 65, usage: '--exclude-from .rsyncignore' },
        { text: '-e', type: 'option', description: '指定远端 shell', priority: 60, usage: 'rsync -e \"ssh -p 22\" src/ host:dest/' },
        { text: '--timeout', type: 'option', description: 'I/O 超时(秒)', priority: 55, usage: '--timeout 30' },
        { text: '--bwlimit', type: 'option', description: '带宽限制(KB/s)', priority: 50, usage: '--bwlimit 1024' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 45 },
    ],
};

export default rsyncCommand;

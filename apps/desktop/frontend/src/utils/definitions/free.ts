
import type { CommandDefinition } from '../types';

/**
 * free 命令 - 查看内存使用
 */
const freeCommand: CommandDefinition = {
    name: 'free',
    description: '显示内存使用情况',
    options: [
        { text: '-h', type: 'option', description: '人类可读单位', priority: 100, usage: 'free -h' },
        { text: '-m', type: 'option', description: '以 MB 显示', priority: 90 },
        { text: '-g', type: 'option', description: '以 GB 显示', priority: 85 },
        { text: '-k', type: 'option', description: '以 KB 显示', priority: 80 },
        { text: '--si', type: 'option', description: '使用 1000 为单位', priority: 75 },
        { text: '-t', type: 'option', description: '显示总计行', priority: 70 },
        { text: '-s', type: 'option', description: '周期刷新(秒)', priority: 65, usage: 'free -s 1' },
        { text: '-w', type: 'option', description: '宽输出', priority: 60 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 45 },
    ],
};

export default freeCommand;


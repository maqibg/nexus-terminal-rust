
import type { CommandDefinition } from '../types';

/**
 * apt-mark 命令定义（Debian/Ubuntu）
 */

const aptMarkCommand: CommandDefinition = {
    name: 'apt-mark',
    description: '标记包为手动/自动/hold',
    options: [
        { text: 'hold', type: 'subcommand', description: '锁定版本（hold）', priority: 100, usage: 'apt-mark hold nginx' },
        { text: 'unhold', type: 'subcommand', description: '取消 hold', priority: 95, usage: 'apt-mark unhold nginx' },
        { text: 'showhold', type: 'subcommand', description: '列出 hold 的包', priority: 90, usage: 'apt-mark showhold' },
        { text: 'manual', type: 'subcommand', description: '标记为手动安装', priority: 85, usage: 'apt-mark manual nginx' },
        { text: 'auto', type: 'subcommand', description: '标记为自动安装', priority: 80, usage: 'apt-mark auto nginx' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        hold: { name: 'hold', description: '锁定版本', options: [] },
        unhold: { name: 'unhold', description: '取消 hold', options: [] },
        showhold: { name: 'showhold', description: '列出 hold 的包', options: [] },
        manual: { name: 'manual', description: '标记为手动', options: [] },
        auto: { name: 'auto', description: '标记为自动', options: [] },
    },
};

export default aptMarkCommand;


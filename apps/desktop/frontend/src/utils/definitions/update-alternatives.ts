
import type { CommandDefinition } from '../types';

/**
 * update-alternatives 命令定义（Debian/Ubuntu）
 */

const updateAlternativesCommand: CommandDefinition = {
    name: 'update-alternatives',
    description: '管理 alternatives（默认程序选择）',
    options: [
        { text: '--install', type: 'option', description: '安装一个 alternative', priority: 95, usage: 'update-alternatives --install /usr/bin/editor editor /usr/bin/vim 100' },
        { text: '--remove', type: 'option', description: '移除一个 alternative', priority: 90, usage: 'update-alternatives --remove editor /usr/bin/vim' },
        { text: '--config', type: 'option', description: '交互式选择', priority: 88, usage: 'update-alternatives --config editor' },
        { text: '--set', type: 'option', description: '设置为指定路径', priority: 86, usage: 'update-alternatives --set editor /usr/bin/vim' },
        { text: '--auto', type: 'option', description: '切回自动模式', priority: 84, usage: 'update-alternatives --auto editor' },
        { text: '--display', type: 'option', description: '显示配置', priority: 82, usage: 'update-alternatives --display editor' },
        { text: '--list', type: 'option', description: '列出候选', priority: 80, usage: 'update-alternatives --list editor' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default updateAlternativesCommand;


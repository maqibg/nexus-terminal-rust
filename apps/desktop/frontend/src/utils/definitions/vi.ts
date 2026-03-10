
import type { CommandDefinition } from '../types';
import vimCommand from './vim';

/**
 * vi 命令 - vim 的别名
 */
const viCommand: CommandDefinition = {
    ...vimCommand,
    name: 'vi',
    description: 'Vim 文本编辑器 (vi)',
};

export default viCommand;



import type { CommandDefinition } from '../types';
import pythonCommand from './python';

/**
 * python3 命令 - python 的变体
 */
const python3Command: CommandDefinition = {
    ...pythonCommand,
    name: 'python3',
    description: 'Python 解释器 (python3)',
};

export default python3Command;



import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * tee 命令定义
 */
const teeCommand: CommandDefinition = {
    name: 'tee',
    description: '把输入写入文件并输出',
    options: [
        { text: '-a', type: 'option', description: '追加', priority: 95, usage: 'tee -a out.log' },
        { text: '-i', type: 'option', description: '忽略中断信号', priority: 80 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId!, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    }
};

export default teeCommand;


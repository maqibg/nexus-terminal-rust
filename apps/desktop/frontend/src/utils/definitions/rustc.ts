
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * rustc 命令定义
 */

const RUSTC_PATH_VALUE_OPTIONS = new Set(['-o', '--out-dir', '-L', '--sysroot']);

const rustcCommand: CommandDefinition = {
    name: 'rustc',
    description: 'Rust 编译器',
    options: [
        { text: '-O', type: 'option', description: '优化（opt-level=2）', priority: 90 },
        { text: '-g', type: 'option', description: '生成调试信息', priority: 88 },
        { text: '--edition', type: 'option', description: '指定 edition', priority: 86, usage: '--edition 2021' },
        { text: '--crate-type', type: 'option', description: 'crate 类型', priority: 84, usage: '--crate-type bin' },
        { text: '--crate-name', type: 'option', description: 'crate 名称', priority: 82 },
        { text: '--emit', type: 'option', description: '输出类型', priority: 80, usage: '--emit asm' },
        { text: '--target', type: 'option', description: '目标 triple', priority: 78, usage: '--target x86_64-unknown-linux-gnu' },
        { text: '-L', type: 'option', description: '库搜索路径', priority: 76, usage: '-L dependency=./target/debug/deps' },
        { text: '--extern', type: 'option', description: '指定外部 crate', priority: 74, usage: '--extern anyhow=path/to/libanyhow.rlib' },
        { text: '-o', type: 'option', description: '输出文件', priority: 72 },
        { text: '--out-dir', type: 'option', description: '输出目录', priority: 70 },
        { text: '--sysroot', type: 'option', description: '指定 sysroot', priority: 68 },
        { text: '--color', type: 'option', description: '颜色输出', priority: 66, usage: '--color always' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 48 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (RUSTC_PATH_VALUE_OPTIONS.has(prevArg)) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    },
};

export default rustcCommand;


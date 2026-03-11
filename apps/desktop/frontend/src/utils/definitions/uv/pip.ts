
import type { CommandDefinition, CompletionContext, CompletionItem } from '../../types';
import { suggestRemotePaths } from './helpers';

const uvPipCompile: CommandDefinition = {
    name: 'compile',
    description: '将 requirements.in 编译为 requirements.txt/pylock.toml',
    options: [
        { text: '-o', type: 'option', description: '输出文件', priority: 95, usage: 'uv pip compile -o requirements.txt requirements.in' },
        { text: '--output-file', type: 'option', description: '输出文件', priority: 95 },
        { text: '--format', type: 'option', description: '输出格式', priority: 90, usage: '--format requirements.txt' },
        { text: '--generate-hashes', type: 'option', description: '生成 hashes', priority: 85 },
        { text: '-c', type: 'option', description: '约束文件', priority: 80, usage: 'uv pip compile -c constraints.txt requirements.in' },
        { text: '--constraints', type: 'option', description: '约束文件', priority: 80 },
        { text: '--group', type: 'option', description: '从 pyproject.toml 选择依赖组', priority: 75, usage: '--group dev' },
        { text: '--all-extras', type: 'option', description: '包含所有 extras', priority: 70 },
        { text: '--no-deps', type: 'option', description: '不解析依赖，仅输出显式包', priority: 65 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['-o', '--output-file', '-c', '--constraints'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        if (!ctx.currentArg.startsWith('-')) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

const uvPipSync: CommandDefinition = {
    name: 'sync',
    description: '根据 requirements.txt/pylock.toml 同步环境',
    options: [
        { text: '-c', type: 'option', description: '约束文件', priority: 85, usage: 'uv pip sync -c constraints.txt requirements.txt' },
        { text: '--constraints', type: 'option', description: '约束文件', priority: 85 },
        { text: '--system', type: 'option', description: '安装到系统 Python', priority: 80 },
        { text: '--target', type: 'option', description: '安装到目录', priority: 75, usage: '--target ./vendor' },
        { text: '--prefix', type: 'option', description: '按 prefix 方式安装', priority: 70, usage: '--prefix ./venv-like' },
        { text: '--dry-run', type: 'option', description: '仅演练，不实际安装', priority: 65 },
        { text: '--strict', type: 'option', description: '校验环境依赖一致性', priority: 60 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['-c', '--constraints', '--target', '--prefix'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        if (!ctx.currentArg.startsWith('-')) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

const uvPipInstall: CommandDefinition = {
    name: 'install',
    description: '向环境中安装包（pip 兼容）',
    options: [
        { text: '-r', type: 'option', description: '从 requirements 安装', priority: 95, usage: 'uv pip install -r requirements.txt' },
        { text: '--requirements', type: 'option', description: '从 requirements 安装', priority: 95 },
        { text: '-e', type: 'option', description: '可编辑安装（本地路径）', priority: 90, usage: 'uv pip install -e .' },
        { text: '--editable', type: 'option', description: '可编辑安装（本地路径）', priority: 90 },
        { text: '--system', type: 'option', description: '安装到系统 Python', priority: 85 },
        { text: '--no-deps', type: 'option', description: '不安装依赖', priority: 80 },
        { text: '-U', type: 'option', description: '允许升级（upgrade）', priority: 75 },
        { text: '--upgrade', type: 'option', description: '允许升级（upgrade）', priority: 75 },
        { text: '-p', type: 'option', description: '指定 Python 解释器', priority: 70, usage: '--python python3.12' },
        { text: '--python', type: 'option', description: '指定 Python 解释器', priority: 70 },
        { text: '--target', type: 'option', description: '安装到目录', priority: 65 },
        { text: '--prefix', type: 'option', description: '按 prefix 方式安装', priority: 60 },
        { text: '--dry-run', type: 'option', description: '仅演练，不实际安装', priority: 55 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['-r', '--requirements', '-e', '--editable', '--target', '--prefix', '-p', '--python'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

export const uvPip: CommandDefinition = {
    name: 'pip',
    description: 'pip 兼容接口管理 Python 包',
    options: [
        { text: 'compile', type: 'subcommand', description: '编译 requirements', priority: 100 },
        { text: 'sync', type: 'subcommand', description: '同步环境', priority: 95 },
        { text: 'install', type: 'subcommand', description: '安装包', priority: 90 },
        { text: 'uninstall', type: 'subcommand', description: '卸载包', priority: 85 },
        { text: 'freeze', type: 'subcommand', description: '按 requirements 格式列出已装包', priority: 80 },
        { text: 'list', type: 'subcommand', description: '列表显示已装包', priority: 78 },
        { text: 'show', type: 'subcommand', description: '显示包信息', priority: 76 },
        { text: 'tree', type: 'subcommand', description: '依赖树', priority: 74 },
        { text: 'check', type: 'subcommand', description: '检查依赖兼容性', priority: 72 },
    ],
    subcommands: {
        compile: uvPipCompile,
        sync: uvPipSync,
        install: uvPipInstall,
        uninstall: { name: 'uninstall', description: '卸载包', options: [] },
        freeze: { name: 'freeze', description: '列出已装包（requirements）', options: [] },
        list: { name: 'list', description: '列出已装包（表格）', options: [] },
        show: { name: 'show', description: '显示包信息', options: [] },
        tree: { name: 'tree', description: '依赖树', options: [] },
        check: { name: 'check', description: '检查依赖', options: [] },
    },
};


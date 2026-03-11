
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * Yarn 命令定义
 * 支持动态读取 package.json 中的 scripts
 */

async function getPackageScripts(ctx: CompletionContext): Promise<CompletionItem[]> {
    if (!ctx.sessionId || !ctx.electronAPI) return [];

    try {
        const result = await ctx.electronAPI.ssh?.executeCommand?.(
            ctx.sessionId,
            `cat package.json 2>/dev/null | grep -A 100 '"scripts"' | head -50`,
            3000
        );

        if (!result?.success || !result.data) {
            return [
                { text: 'dev', type: 'subcommand', description: '开发模式', priority: 100, matchPart: '', restPart: 'dev' },
                { text: 'start', type: 'subcommand', description: '启动应用', priority: 95, matchPart: '', restPart: 'start' },
                { text: 'build', type: 'subcommand', description: '构建项目', priority: 90, matchPart: '', restPart: 'build' },
                { text: 'test', type: 'subcommand', description: '运行测试', priority: 85, matchPart: '', restPart: 'test' }
            ];
        }

        const scripts: CompletionItem[] = [];
        const lines = result.data.split('\n');
        for (const line of lines) {
            const match = line.match(/"([^"]+)":\s*"([^"]+)"/);
            if (match && match[1] !== 'scripts') {
                scripts.push({
                    text: match[1],
                    type: 'subcommand' as const,
                    description: match[2].substring(0, 60),
                    priority: 80,
                    matchPart: '',
                    restPart: match[1]
                });
            }
        }

        return scripts.length > 0 ? scripts : [
            { text: 'dev', type: 'subcommand', description: '开发模式', priority: 100, matchPart: '', restPart: 'dev' },
            { text: 'start', type: 'subcommand', description: '启动应用', priority: 95, matchPart: '', restPart: 'start' }
        ];
    } catch {
        return [];
    }
}

const yarnAdd: CommandDefinition = {
    name: 'add',
    description: '添加依赖',
    options: [
        { text: '-D', type: 'option', description: '开发依赖', priority: 100, usage: 'yarn add -D pkg' },
        { text: '--dev', type: 'option', description: '开发依赖', priority: 98 },
        { text: '-E', type: 'option', description: '精确版本', priority: 90 },
        { text: '--exact', type: 'option', description: '精确版本', priority: 90 },
        { text: '-T', type: 'option', description: '使用 ~ 版本', priority: 85 },
        { text: '--tilde', type: 'option', description: '使用 ~ 版本', priority: 85 },
        { text: '-P', type: 'option', description: 'peer 依赖', priority: 80 },
        { text: '--peer', type: 'option', description: 'peer 依赖', priority: 80 },
        { text: '--ignore-workspace-root-check', type: 'option', description: '允许在 workspace root 执行', priority: 70 },
    ],
};

const yarnRemove: CommandDefinition = {
    name: 'remove',
    description: '移除依赖',
    options: [],
};

const yarnInstall: CommandDefinition = {
    name: 'install',
    description: '安装依赖',
    options: [
        { text: '--frozen-lockfile', type: 'option', description: '锁定 lockfile', priority: 95 },
        { text: '--immutable', type: 'option', description: '锁定 lockfile (Berry)', priority: 90 },
        { text: '--check-files', type: 'option', description: '检查文件存在性', priority: 80 },
    ],
};

const yarnRun: CommandDefinition = {
    name: 'run',
    description: '运行脚本',
    options: [],
    generate: getPackageScripts,
};

const yarnInit: CommandDefinition = {
    name: 'init',
    description: '初始化项目',
    options: [
        { text: '-y', type: 'option', description: '使用默认值', priority: 100, usage: 'yarn init -y' },
        { text: '--yes', type: 'option', description: '使用默认值', priority: 100 },
        { text: '--private', type: 'option', description: '设置 private=true', priority: 90 },
    ],
};

const yarnWhy: CommandDefinition = {
    name: 'why',
    description: '查看依赖原因',
    options: [
        { text: '--recursive', type: 'option', description: '递归分析', priority: 80 },
    ],
};

const yarnCache: CommandDefinition = {
    name: 'cache',
    description: '管理缓存',
    options: [
        { text: 'clean', type: 'subcommand', description: '清理缓存', priority: 100, usage: 'yarn cache clean' },
        { text: 'dir', type: 'subcommand', description: '显示缓存目录', priority: 80, usage: 'yarn cache dir' },
        { text: 'list', type: 'subcommand', description: '列出缓存条目', priority: 75, usage: 'yarn cache list' },
    ],
    subcommands: {
        clean: { name: 'clean', description: '清理缓存', options: [] },
        dir: { name: 'dir', description: '显示缓存目录', options: [] },
        list: { name: 'list', description: '列出缓存条目', options: [] },
    },
};

const yarnCommand: CommandDefinition = {
    name: 'yarn',
    description: 'Yarn 包管理器',
    options: [
        { text: 'install', type: 'subcommand', description: '安装依赖', priority: 100, usage: 'yarn install' },
        { text: 'add', type: 'subcommand', description: '添加依赖', priority: 95, usage: 'yarn add pkg' },
        { text: 'remove', type: 'subcommand', description: '移除依赖', priority: 90, usage: 'yarn remove pkg' },
        { text: 'run', type: 'subcommand', description: '运行脚本', priority: 90, usage: 'yarn run dev' },
        { text: 'init', type: 'subcommand', description: '初始化项目', priority: 80, usage: 'yarn init -y' },
        { text: 'why', type: 'subcommand', description: '查看依赖原因', priority: 70 },
        { text: 'cache', type: 'subcommand', description: '管理缓存', priority: 65, usage: 'yarn cache clean' },
        { text: '--version', type: 'option', description: '显示版本', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        'install': yarnInstall,
        'add': yarnAdd,
        'remove': yarnRemove,
        'run': yarnRun,
        'init': yarnInit,
        'why': yarnWhy,
        'cache': yarnCache,
    },
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        // yarn <script> 形式运行 scripts
        if (ctx.currentArgIndex === 1 && !ctx.currentArg.startsWith('-')) {
            const scripts = await getPackageScripts(ctx);
            return scripts.filter((item) => !ctx.currentArg || item.text.startsWith(ctx.currentArg));
        }
        return [];
    }
};

export default yarnCommand;

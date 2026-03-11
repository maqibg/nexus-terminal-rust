
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * NPM 命令定义
 * 支持动态读取 package.json 中的脚本
 */

// 动态读取 package.json 脚本
async function getNpmScripts(ctx: CompletionContext): Promise<CompletionItem[]> {
    if (!ctx.sessionId || !ctx.electronAPI) return [];

    try {
        const result = await ctx.electronAPI.ssh?.executeCommand?.(
            ctx.sessionId,
            `cat package.json 2>/dev/null | grep -A 100 '"scripts"' | head -50`,
            3000
        );

        if (!result?.success || !result.data) {
            // 返回常见脚本作为后备
            return [
                { text: 'dev', type: 'hint', description: '开发模式', priority: 100, matchPart: '', restPart: 'dev' },
                { text: 'start', type: 'hint', description: '启动应用', priority: 95, matchPart: '', restPart: 'start' },
                { text: 'build', type: 'hint', description: '构建项目', priority: 90, matchPart: '', restPart: 'build' },
                { text: 'test', type: 'hint', description: '运行测试', priority: 85, matchPart: '', restPart: 'test' }
            ];
        }

        // 解析脚本名称
        const scripts: CompletionItem[] = [];
        const lines = result.data.split('\n');
        for (const line of lines) {
            const match = line.match(/"([^"]+)":\s*"([^"]+)"/);
            if (match && match[1] !== 'scripts') {
                scripts.push({
                    text: match[1],
                    type: 'hint' as const,
                    description: match[2].substring(0, 50),
                    priority: 80,
                    matchPart: '',
                    restPart: match[1]
                });
            }
        }
        return scripts.length > 0 ? scripts : [
            { text: 'dev', type: 'hint', description: '开发模式', priority: 100, matchPart: '', restPart: 'dev' },
            { text: 'start', type: 'hint', description: '启动应用', priority: 95, matchPart: '', restPart: 'start' }
        ];
    } catch {
        return [];
    }
}

// npm run 子命令
const npmRun: CommandDefinition = {
    name: 'run',
    description: '运行脚本',
    options: [],
    generate: getNpmScripts
};

// npm install 子命令
const npmInstall: CommandDefinition = {
    name: 'install',
    description: '安装包',
    options: [
        { text: '-D', type: 'option', description: '开发依赖 (--save-dev)', priority: 100, usage: 'npm i -D pkg' },
        { text: '-g', type: 'option', description: '全局安装', priority: 95, usage: 'npm i -g pkg' },
        { text: '--save-dev', type: 'option', description: '开发依赖', priority: 90 },
        { text: '--save-exact', type: 'option', description: '精确版本', priority: 85 },
        { text: '--legacy-peer-deps', type: 'option', description: '忽略 peer 依赖', priority: 80 }
    ]
};

// npm uninstall 子命令
const npmUninstall: CommandDefinition = {
    name: 'uninstall',
    description: '卸载包',
    options: [
        { text: '-g', type: 'option', description: '全局卸载', priority: 90 }
    ]
};

// npm init 子命令
const npmInit: CommandDefinition = {
    name: 'init',
    description: '初始化项目',
    options: [
        { text: '-y', type: 'option', description: '使用默认值', priority: 100 },
        { text: 'vite', type: 'hint', description: 'Vite 项目', priority: 90, usage: 'npm init vite@latest' },
        { text: 'vue', type: 'hint', description: 'Vue 项目', priority: 85, usage: 'npm init vue@latest' },
        { text: 'react-app', type: 'hint', description: 'React 项目', priority: 85 }
    ]
};

// npm exec/npx 子命令
const npmExec: CommandDefinition = {
    name: 'exec',
    description: '执行包命令',
    options: [
        { text: 'create-vite', type: 'hint', description: '创建 Vite 项目', priority: 90 },
        { text: 'create-next-app', type: 'hint', description: '创建 Next.js 项目', priority: 85 },
        { text: 'create-react-app', type: 'hint', description: '创建 React 项目', priority: 85 },
        { text: 'prisma', type: 'hint', description: 'Prisma ORM', priority: 80 },
        { text: 'tsc', type: 'hint', description: 'TypeScript 编译器', priority: 80 }
    ]
};

const npmStart: CommandDefinition = {
    name: 'start',
    description: '启动应用',
    options: [
        { text: '--silent', type: 'option', description: '静默输出', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

const npmTest: CommandDefinition = {
    name: 'test',
    description: '运行测试',
    options: [
        { text: '--silent', type: 'option', description: '静默输出', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

const npmBuild: CommandDefinition = {
    name: 'build',
    description: '构建项目',
    options: [
        { text: '--silent', type: 'option', description: '静默输出', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

const npmUpdate: CommandDefinition = {
    name: 'update',
    description: '更新包',
    options: [
        { text: '-g', type: 'option', description: '全局更新', priority: 90, usage: 'npm update -g' },
        { text: '--global', type: 'option', description: '全局更新', priority: 90 },
        { text: '--depth', type: 'option', description: '依赖深度', priority: 70, usage: '--depth 0' },
    ],
};

const npmList: CommandDefinition = {
    name: 'list',
    description: '列出已安装包',
    options: [
        { text: '-g', type: 'option', description: '全局列表', priority: 90, usage: 'npm list -g --depth 0' },
        { text: '--global', type: 'option', description: '全局列表', priority: 90 },
        { text: '--depth', type: 'option', description: '依赖深度', priority: 85, usage: '--depth 0' },
        { text: '--json', type: 'option', description: 'JSON 输出', priority: 70 },
    ],
};

const npmOutdated: CommandDefinition = {
    name: 'outdated',
    description: '检查过期包',
    options: [
        { text: '-g', type: 'option', description: '全局检查', priority: 85 },
        { text: '--global', type: 'option', description: '全局检查', priority: 85 },
        { text: '--json', type: 'option', description: 'JSON 输出', priority: 70 },
    ],
};

const npmPublish: CommandDefinition = {
    name: 'publish',
    description: '发布包',
    options: [
        { text: '--access', type: 'option', description: '访问级别', priority: 90, usage: '--access public' },
        { text: '--tag', type: 'option', description: '发布 tag', priority: 85, usage: '--tag latest' },
        { text: '--dry-run', type: 'option', description: '演练，不实际发布', priority: 75 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

const npmCache: CommandDefinition = {
    name: 'cache',
    description: '管理缓存',
    options: [
        { text: 'clean', type: 'subcommand', description: '清理缓存', priority: 100, usage: 'npm cache clean --force' },
        { text: 'verify', type: 'subcommand', description: '校验缓存', priority: 80, usage: 'npm cache verify' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        clean: {
            name: 'clean',
            description: '清理缓存',
            options: [
                { text: '--force', type: 'option', description: '强制执行', priority: 95 },
            ],
        },
        verify: { name: 'verify', description: '校验缓存', options: [] },
    },
};

// 主 npm 命令
const npmCommand: CommandDefinition = {
    name: 'npm',
    description: 'Node.js 包管理器',
    options: [
        { text: 'install', type: 'subcommand', description: '安装依赖', priority: 100, usage: 'npm install' },
        { text: 'i', type: 'subcommand', description: '安装依赖 (简写)', priority: 95, usage: 'npm i pkg' },
        { text: 'run', type: 'subcommand', description: '运行脚本', priority: 95, usage: 'npm run dev' },
        { text: 'start', type: 'subcommand', description: '启动应用', priority: 90, usage: 'npm start' },
        { text: 'test', type: 'subcommand', description: '运行测试', priority: 85, usage: 'npm test' },
        { text: 'build', type: 'subcommand', description: '构建项目', priority: 85, usage: 'npm run build' },
        { text: 'init', type: 'subcommand', description: '初始化项目', priority: 80, usage: 'npm init -y' },
        { text: 'uninstall', type: 'subcommand', description: '卸载包', priority: 75, usage: 'npm uninstall pkg' },
        { text: 'update', type: 'subcommand', description: '更新包', priority: 70, usage: 'npm update' },
        { text: 'list', type: 'subcommand', description: '列出已安装包', priority: 65, usage: 'npm list' },
        { text: 'outdated', type: 'subcommand', description: '检查过期包', priority: 60, usage: 'npm outdated' },
        { text: 'publish', type: 'subcommand', description: '发布包', priority: 55, usage: 'npm publish' },
        { text: 'exec', type: 'subcommand', description: '执行包命令', priority: 50, usage: 'npm exec -- pkg' },
        { text: 'cache', type: 'subcommand', description: '管理缓存', priority: 45, usage: 'npm cache clean --force' }
    ],
    subcommands: {
        'install': npmInstall,
        'i': npmInstall,
        'run': npmRun,
        'start': npmStart,
        'test': npmTest,
        'build': npmBuild,
        'uninstall': npmUninstall,
        'update': npmUpdate,
        'list': npmList,
        'outdated': npmOutdated,
        'publish': npmPublish,
        'init': npmInit,
        'exec': npmExec,
        'cache': npmCache,
    }
};

export default npmCommand;

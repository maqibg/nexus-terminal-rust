
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { UV_ROOT_PATH_VALUE_OPTIONS, suggestRemotePaths } from './uv/helpers';
import { uvCache } from './uv/cache';
import { uvPip } from './uv/pip';
import { uvPython } from './uv/python';
import { uvSelf } from './uv/self';
import { uvTool } from './uv/tool';

/**
 * uv 命令定义（Python 包/项目管理）
 * 参考：`uv --help`
 */



const uvRun: CommandDefinition = {
    name: 'run',
    description: '运行命令或脚本（项目环境）',
    options: [
        { text: '-m', type: 'option', description: '运行模块', priority: 95, usage: 'uv run -m http.server' },
        { text: '-s', type: 'option', description: '运行脚本路径', priority: 90, usage: 'uv run -s ./main.py' },
        { text: '--script', type: 'option', description: '运行脚本路径', priority: 90 },
        { text: '--env-file', type: 'option', description: '加载 .env 文件', priority: 85 },
        { text: '--with', type: 'option', description: '临时安装并运行（包）', priority: 80, usage: '--with ruff' },
        { text: '--with-requirements', type: 'option', description: '临时安装并运行（requirements）', priority: 78 },
        { text: '--isolated', type: 'option', description: '隔离虚拟环境运行', priority: 75 },
        { text: '--active', type: 'option', description: '优先使用激活的 venv', priority: 70 },
        { text: '--no-sync', type: 'option', description: '不自动 sync 环境', priority: 65 },
        { text: '-p', type: 'option', description: '指定 Python 解释器', priority: 60, usage: '--python python3.12' },
        { text: '--python', type: 'option', description: '指定 Python 解释器', priority: 60 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['-s', '--script', '--env-file', '--with-requirements', '-p', '--python'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

const uvVenv: CommandDefinition = {
    name: 'venv',
    description: '创建虚拟环境',
    options: [
        { text: '-p', type: 'option', description: '指定 Python 解释器', priority: 90, usage: 'uv venv -p python3.12 .venv' },
        { text: '--python', type: 'option', description: '指定 Python 解释器', priority: 90 },
        { text: '--seed', type: 'option', description: '安装 seed 包（pip/setuptools/wheel）', priority: 80 },
        { text: '--allow-existing', type: 'option', description: '允许目标已存在', priority: 70 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['-p', '--python'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        if (!ctx.currentArg.startsWith('-')) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

const uvInit: CommandDefinition = {
    name: 'init',
    description: '初始化项目（pyproject.toml）',
    options: [
        { text: '--name', type: 'option', description: '项目名', priority: 95 },
        { text: '--package', type: 'option', description: '初始化为可发布包', priority: 90 },
        { text: '--app', type: 'option', description: '应用项目模板', priority: 85 },
        { text: '--lib', type: 'option', description: '库项目模板', priority: 80 },
        { text: '--script', type: 'option', description: '脚本模板', priority: 75 },
        { text: '--vcs', type: 'option', description: '初始化版本控制', priority: 70, usage: '--vcs git' },
        { text: '--build-backend', type: 'option', description: '构建后端', priority: 65, usage: '--build-backend setuptools' },
        { text: '--no-readme', type: 'option', description: '不生成 README.md', priority: 60 },
        { text: '-p', type: 'option', description: '指定 Python 解释器', priority: 55 },
        { text: '--python', type: 'option', description: '指定 Python 解释器', priority: 55 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['-p', '--python'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

const uvAdd: CommandDefinition = {
    name: 'add',
    description: '添加依赖到项目',
    options: [
        { text: '-r', type: 'option', description: '从 requirements 添加', priority: 95, usage: 'uv add -r requirements.txt' },
        { text: '--requirements', type: 'option', description: '从 requirements 添加', priority: 95 },
        { text: '--dev', type: 'option', description: '添加到 dev 组', priority: 90 },
        { text: '--group', type: 'option', description: '指定依赖组', priority: 85, usage: '--group dev' },
        { text: '--optional', type: 'option', description: '指定 extra（可选依赖）', priority: 80, usage: '--optional docs' },
        { text: '--editable', type: 'option', description: '以 editable 方式添加', priority: 75 },
        { text: '--no-sync', type: 'option', description: '不自动 sync', priority: 70 },
        { text: '--locked', type: 'option', description: '锁定 uv.lock 不变', priority: 65 },
        { text: '--frozen', type: 'option', description: '不更新 lockfile', priority: 60 },
        { text: '--package', type: 'option', description: '指定工作区包', priority: 55 },
        { text: '--script', type: 'option', description: '将依赖添加到脚本', priority: 50 },
        { text: '-p', type: 'option', description: '指定 Python 解释器', priority: 45 },
        { text: '--python', type: 'option', description: '指定 Python 解释器', priority: 45 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (['-r', '--requirements', '--script', '-p', '--python'].includes(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

const uvRemove: CommandDefinition = {
    name: 'remove',
    description: '从项目移除依赖',
    options: [
        { text: '--dev', type: 'option', description: '从 dev 组移除', priority: 90 },
        { text: '--group', type: 'option', description: '指定依赖组', priority: 85, usage: '--group dev' },
        { text: '--optional', type: 'option', description: '指定 extra（可选依赖）', priority: 80, usage: '--optional docs' },
        { text: '--no-sync', type: 'option', description: '不自动 sync', priority: 75 },
        { text: '--locked', type: 'option', description: '锁定 uv.lock 不变', priority: 70 },
        { text: '--frozen', type: 'option', description: '不更新 lockfile', priority: 65 },
        { text: '--package', type: 'option', description: '指定工作区包', priority: 60 },
        { text: '--script', type: 'option', description: '从脚本移除依赖', priority: 55 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '--script') {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

const uvCommand: CommandDefinition = {
    name: 'uv',
    description: '极快的 Python 包/项目管理器',
    options: [
        { text: 'run', type: 'subcommand', description: '运行命令或脚本', priority: 100 },
        { text: 'init', type: 'subcommand', description: '初始化项目', priority: 98 },
        { text: 'add', type: 'subcommand', description: '添加依赖', priority: 96 },
        { text: 'remove', type: 'subcommand', description: '移除依赖', priority: 94 },
        { text: 'version', type: 'subcommand', description: '读取/更新项目版本', priority: 92 },
        { text: 'sync', type: 'subcommand', description: '同步项目环境', priority: 90 },
        { text: 'lock', type: 'subcommand', description: '生成/更新 lockfile', priority: 88 },
        { text: 'export', type: 'subcommand', description: '导出 lockfile', priority: 86 },
        { text: 'tree', type: 'subcommand', description: '依赖树', priority: 84 },
        { text: 'tool', type: 'subcommand', description: '安装/运行工具', priority: 82 },
        { text: 'python', type: 'subcommand', description: '管理 Python 版本', priority: 80 },
        { text: 'pip', type: 'subcommand', description: 'pip 兼容接口', priority: 78 },
        { text: 'venv', type: 'subcommand', description: '创建虚拟环境', priority: 76 },
        { text: 'build', type: 'subcommand', description: '构建发行包', priority: 74 },
        { text: 'publish', type: 'subcommand', description: '发布到索引', priority: 72 },
        { text: 'cache', type: 'subcommand', description: '管理缓存', priority: 70 },
        { text: 'self', type: 'subcommand', description: '管理 uv 本体', priority: 68 },
        { text: 'help', type: 'subcommand', description: '显示帮助', priority: 66 },

        { text: '-q', type: 'option', description: '更安静的输出（可重复）', priority: 60, repeatable: true },
        { text: '--quiet', type: 'option', description: '更安静的输出（可重复）', priority: 60, repeatable: true },
        { text: '-v', type: 'option', description: '更详细的输出（可重复）', priority: 58, repeatable: true },
        { text: '--verbose', type: 'option', description: '更详细的输出（可重复）', priority: 58, repeatable: true },
        { text: '--offline', type: 'option', description: '禁用网络访问', priority: 56 },
        { text: '--no-progress', type: 'option', description: '隐藏进度输出', priority: 54 },
        { text: '--directory', type: 'option', description: '切换到目录后执行', priority: 52, usage: '--directory /path/to/project' },
        { text: '--project', type: 'option', description: '指定项目目录', priority: 50, usage: '--project /path/to/project' },
        { text: '--config-file', type: 'option', description: '指定 uv.toml', priority: 48, usage: '--config-file ./uv.toml' },
        { text: '--no-config', type: 'option', description: '不发现配置文件', priority: 46 },
        { text: '-n', type: 'option', description: '不读写缓存（no-cache）', priority: 44 },
        { text: '--no-cache', type: 'option', description: '不读写缓存', priority: 44 },
        { text: '--cache-dir', type: 'option', description: '指定缓存目录', priority: 42 },
        { text: '-h', type: 'option', description: '显示帮助', priority: 40 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 40 },
        { text: '-V', type: 'option', description: '显示 uv 版本', priority: 38 },
        { text: '--version', type: 'option', description: '显示 uv 版本', priority: 38 },
    ],
    subcommands: {
        run: uvRun,
        init: uvInit,
        add: uvAdd,
        remove: uvRemove,
        version: { name: 'version', description: '读取/更新项目版本', options: [] },
        sync: { name: 'sync', description: '同步项目环境', options: [] },
        lock: { name: 'lock', description: '更新 lockfile', options: [] },
        export: { name: 'export', description: '导出 lockfile', options: [] },
        tree: { name: 'tree', description: '显示依赖树', options: [] },
        tool: uvTool,
        python: uvPython,
        pip: uvPip,
        venv: uvVenv,
        build: { name: 'build', description: '构建分发包', options: [] },
        publish: { name: 'publish', description: '上传分发包', options: [] },
        cache: uvCache,
        self: uvSelf,
        help: { name: 'help', description: '显示帮助', options: [] },
    },
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (UV_ROOT_PATH_VALUE_OPTIONS.has(prevArg)) {
            return suggestRemotePaths(ctx);
        }
        return [];
    },
};

export default uvCommand;


import type { CommandDefinition } from '../types';

/**
 * pnpm 命令定义（Node.js 包管理器）
 */

const pnpmAdd: CommandDefinition = {
    name: 'add',
    description: '添加依赖',
    options: [
        { text: '-D', type: 'option', description: '开发依赖', priority: 100, usage: 'pnpm add -D pkg' },
        { text: '--save-dev', type: 'option', description: '开发依赖', priority: 95 },
        { text: '-g', type: 'option', description: '全局安装', priority: 90, usage: 'pnpm add -g pkg' },
        { text: '--global', type: 'option', description: '全局安装', priority: 90 },
        { text: '-w', type: 'option', description: '在工作区根安装（workspace root）', priority: 85, usage: 'pnpm add -w pkg' },
        { text: '--workspace-root', type: 'option', description: '在工作区根安装', priority: 85 },
    ],
};

const pnpmInstall: CommandDefinition = {
    name: 'install',
    description: '安装依赖',
    options: [
        { text: '--frozen-lockfile', type: 'option', description: '锁定 lockfile 不变', priority: 90 },
        { text: '--prefer-offline', type: 'option', description: '优先离线缓存', priority: 80 },
        { text: '--offline', type: 'option', description: '离线模式', priority: 75 },
    ],
};

const pnpmRemove: CommandDefinition = {
    name: 'remove',
    description: '移除依赖',
    options: [
        { text: '-g', type: 'option', description: '全局卸载', priority: 90 },
        { text: '--global', type: 'option', description: '全局卸载', priority: 90 },
    ],
};

const pnpmRun: CommandDefinition = {
    name: 'run',
    description: '运行脚本',
    options: [
        { text: 'dev', type: 'hint', description: '开发模式', priority: 95, usage: 'pnpm run dev' },
        { text: 'build', type: 'hint', description: '构建项目', priority: 90, usage: 'pnpm run build' },
        { text: 'test', type: 'hint', description: '运行测试', priority: 85, usage: 'pnpm run test' },
        { text: 'lint', type: 'hint', description: '代码检查', priority: 80, usage: 'pnpm run lint' },
    ],
};

const pnpmStore: CommandDefinition = {
    name: 'store',
    description: '管理 store',
    options: [
        { text: 'path', type: 'subcommand', description: '显示 store 路径', priority: 100, usage: 'pnpm store path' },
        { text: 'prune', type: 'subcommand', description: '清理未引用包', priority: 90, usage: 'pnpm store prune' },
    ],
    subcommands: {
        path: { name: 'path', description: '显示 store 路径', options: [] },
        prune: { name: 'prune', description: '清理未引用包', options: [] },
    },
};

const pnpmCommand: CommandDefinition = {
    name: 'pnpm',
    description: 'Node.js 包管理器（pnpm）',
    options: [
        { text: 'install', type: 'subcommand', description: '安装依赖', priority: 100, usage: 'pnpm install' },
        { text: 'i', type: 'subcommand', description: '安装依赖（简写）', priority: 95, usage: 'pnpm i' },
        { text: 'add', type: 'subcommand', description: '添加依赖', priority: 95, usage: 'pnpm add pkg' },
        { text: 'remove', type: 'subcommand', description: '移除依赖', priority: 90, usage: 'pnpm remove pkg' },
        { text: 'update', type: 'subcommand', description: '更新依赖', priority: 85, usage: 'pnpm update' },
        { text: 'run', type: 'subcommand', description: '运行脚本', priority: 80, usage: 'pnpm run dev' },
        { text: 'exec', type: 'subcommand', description: '执行本地 bin', priority: 75, usage: 'pnpm exec eslint .' },
        { text: 'dlx', type: 'subcommand', description: '临时执行包命令', priority: 70, usage: 'pnpm dlx create-vite' },
        { text: 'store', type: 'subcommand', description: '管理 store', priority: 65, usage: 'pnpm store prune' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        install: pnpmInstall,
        i: pnpmInstall,
        add: pnpmAdd,
        remove: pnpmRemove,
        update: { name: 'update', description: '更新依赖', options: [] },
        run: pnpmRun,
        exec: { name: 'exec', description: '执行本地 bin', options: [] },
        dlx: { name: 'dlx', description: '临时执行包命令', options: [] },
        store: pnpmStore,
    },
};

export default pnpmCommand;



import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * cargo 命令定义
 */

const CARGO_PATH_VALUE_OPTIONS = new Set(['--manifest-path', '--target-dir']);

const cargoCommand: CommandDefinition = {
    name: 'cargo',
    description: 'Rust 包管理与构建工具',
    options: [
        { text: 'build', type: 'subcommand', description: '构建', priority: 100, usage: 'cargo build' },
        { text: 'run', type: 'subcommand', description: '运行', priority: 98, usage: 'cargo run' },
        { text: 'test', type: 'subcommand', description: '测试', priority: 96, usage: 'cargo test' },
        { text: 'check', type: 'subcommand', description: '快速检查', priority: 94, usage: 'cargo check' },
        { text: 'fmt', type: 'subcommand', description: '格式化（rustfmt）', priority: 92, usage: 'cargo fmt' },
        { text: 'clippy', type: 'subcommand', description: '静态检查（clippy）', priority: 90, usage: 'cargo clippy' },
        { text: 'doc', type: 'subcommand', description: '生成文档', priority: 88, usage: 'cargo doc --open' },
        { text: 'bench', type: 'subcommand', description: '基准测试', priority: 86 },
        { text: 'clean', type: 'subcommand', description: '清理产物', priority: 84 },
        { text: 'new', type: 'subcommand', description: '创建新项目', priority: 82, usage: 'cargo new my-app' },
        { text: 'init', type: 'subcommand', description: '初始化项目', priority: 80, usage: 'cargo init' },
        { text: 'update', type: 'subcommand', description: '更新依赖锁', priority: 78, usage: 'cargo update' },
        { text: 'install', type: 'subcommand', description: '安装二进制', priority: 76, usage: 'cargo install ripgrep' },
        { text: 'uninstall', type: 'subcommand', description: '卸载二进制', priority: 74 },
        { text: 'publish', type: 'subcommand', description: '发布到 crates.io', priority: 72 },
        { text: 'login', type: 'subcommand', description: '登录 crates.io', priority: 70 },
        { text: 'logout', type: 'subcommand', description: '退出登录', priority: 68 },
        { text: 'search', type: 'subcommand', description: '搜索 crate', priority: 66, usage: 'cargo search serde' },
        { text: 'tree', type: 'subcommand', description: '依赖树', priority: 64, usage: 'cargo tree' },
        { text: 'metadata', type: 'subcommand', description: '输出元数据（JSON）', priority: 62 },
        { text: 'vendor', type: 'subcommand', description: 'vendor 依赖', priority: 60 },
        { text: 'fetch', type: 'subcommand', description: '预取依赖', priority: 58 },
        { text: 'fix', type: 'subcommand', description: '自动修复部分问题', priority: 56 },
        { text: 'add', type: 'subcommand', description: '添加依赖（cargo-edit）', priority: 54, usage: 'cargo add anyhow' },
        { text: 'remove', type: 'subcommand', description: '移除依赖（cargo-edit）', priority: 52, usage: 'cargo remove anyhow' },

        { text: '--release', type: 'option', description: 'Release 模式', priority: 80 },
        { text: '--features', type: 'option', description: '启用 features', priority: 78, usage: '--features serde' },
        { text: '--all-features', type: 'option', description: '启用所有 features', priority: 76 },
        { text: '--no-default-features', type: 'option', description: '禁用默认 features', priority: 74 },
        { text: '--workspace', type: 'option', description: '对整个 workspace 生效', priority: 72 },
        { text: '-p', type: 'option', description: '指定 package', priority: 70, usage: '-p my-crate' },
        { text: '--package', type: 'option', description: '指定 package', priority: 70 },
        { text: '--manifest-path', type: 'option', description: '指定 Cargo.toml 路径', priority: 68, usage: '--manifest-path ./Cargo.toml' },
        { text: '--target', type: 'option', description: '指定 target triple', priority: 66, usage: '--target x86_64-unknown-linux-gnu' },
        { text: '--target-dir', type: 'option', description: '指定 target 输出目录', priority: 64 },
        { text: '--locked', type: 'option', description: '锁定 Cargo.lock 不变', priority: 62 },
        { text: '--frozen', type: 'option', description: '离线并锁定 lockfile', priority: 60 },
        { text: '--offline', type: 'option', description: '离线模式', priority: 58 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 48 },
    ],
    subcommands: {
        build: { name: 'build', description: '构建', options: [] },
        run: { name: 'run', description: '运行', options: [] },
        test: { name: 'test', description: '测试', options: [] },
        check: { name: 'check', description: '检查', options: [] },
        fmt: { name: 'fmt', description: '格式化', options: [] },
        clippy: { name: 'clippy', description: 'clippy 检查', options: [] },
        doc: { name: 'doc', description: '生成文档', options: [] },
        bench: { name: 'bench', description: '基准测试', options: [] },
        clean: { name: 'clean', description: '清理', options: [] },
        new: { name: 'new', description: '新建项目', options: [] },
        init: { name: 'init', description: '初始化项目', options: [] },
        update: { name: 'update', description: '更新依赖', options: [] },
        install: { name: 'install', description: '安装二进制', options: [] },
        uninstall: { name: 'uninstall', description: '卸载二进制', options: [] },
        publish: { name: 'publish', description: '发布', options: [] },
        login: { name: 'login', description: '登录', options: [] },
        logout: { name: 'logout', description: '退出登录', options: [] },
        search: { name: 'search', description: '搜索 crate', options: [] },
        tree: { name: 'tree', description: '依赖树', options: [] },
        metadata: { name: 'metadata', description: '元数据', options: [] },
        vendor: { name: 'vendor', description: 'vendor 依赖', options: [] },
        fetch: { name: 'fetch', description: '预取依赖', options: [] },
        fix: { name: 'fix', description: '自动修复', options: [] },
        add: { name: 'add', description: '添加依赖（cargo-edit）', options: [] },
        remove: { name: 'remove', description: '移除依赖（cargo-edit）', options: [] },
    },
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (CARGO_PATH_VALUE_OPTIONS.has(prevArg)) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    },
};

export default cargoCommand;


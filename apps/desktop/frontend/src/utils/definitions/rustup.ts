
import type { CommandDefinition } from '../types';

/**
 * rustup 命令定义
 */

const rustupToolchain: CommandDefinition = {
    name: 'toolchain',
    description: '管理 toolchain',
    options: [
        { text: 'list', type: 'subcommand', description: '列出 toolchain', priority: 100, usage: 'rustup toolchain list' },
        { text: 'install', type: 'subcommand', description: '安装 toolchain', priority: 95, usage: 'rustup toolchain install stable' },
        { text: 'uninstall', type: 'subcommand', description: '卸载 toolchain', priority: 90 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出 toolchain', options: [] },
        install: { name: 'install', description: '安装 toolchain', options: [] },
        uninstall: { name: 'uninstall', description: '卸载 toolchain', options: [] },
    },
};

const rustupTarget: CommandDefinition = {
    name: 'target',
    description: '管理 target',
    options: [
        { text: 'list', type: 'subcommand', description: '列出 target', priority: 100, usage: 'rustup target list' },
        { text: 'add', type: 'subcommand', description: '添加 target', priority: 95, usage: 'rustup target add x86_64-unknown-linux-gnu' },
        { text: 'remove', type: 'subcommand', description: '移除 target', priority: 90 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出 target', options: [] },
        add: { name: 'add', description: '添加 target', options: [] },
        remove: { name: 'remove', description: '移除 target', options: [] },
    },
};

const rustupComponent: CommandDefinition = {
    name: 'component',
    description: '管理组件（rustfmt/clippy 等）',
    options: [
        { text: 'list', type: 'subcommand', description: '列出组件', priority: 100, usage: 'rustup component list' },
        { text: 'add', type: 'subcommand', description: '安装组件', priority: 95, usage: 'rustup component add rustfmt' },
        { text: 'remove', type: 'subcommand', description: '卸载组件', priority: 90 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出组件', options: [] },
        add: { name: 'add', description: '安装组件', options: [] },
        remove: { name: 'remove', description: '卸载组件', options: [] },
    },
};

const rustupOverride: CommandDefinition = {
    name: 'override',
    description: '目录级 toolchain 覆盖',
    options: [
        { text: 'list', type: 'subcommand', description: '列出覆盖', priority: 100 },
        { text: 'set', type: 'subcommand', description: '设置覆盖', priority: 95, usage: 'rustup override set stable' },
        { text: 'unset', type: 'subcommand', description: '取消覆盖', priority: 90, usage: 'rustup override unset' },
    ],
    subcommands: {
        list: { name: 'list', description: '列出覆盖', options: [] },
        set: { name: 'set', description: '设置覆盖', options: [] },
        unset: { name: 'unset', description: '取消覆盖', options: [] },
    },
};

const rustupCommand: CommandDefinition = {
    name: 'rustup',
    description: 'Rust toolchain 管理器',
    options: [
        { text: 'show', type: 'subcommand', description: '显示当前配置', priority: 100, usage: 'rustup show' },
        { text: 'update', type: 'subcommand', description: '更新 toolchain', priority: 98, usage: 'rustup update' },
        { text: 'default', type: 'subcommand', description: '设置默认 toolchain', priority: 96, usage: 'rustup default stable' },
        { text: 'toolchain', type: 'subcommand', description: '管理 toolchain', priority: 94 },
        { text: 'target', type: 'subcommand', description: '管理 target', priority: 92 },
        { text: 'component', type: 'subcommand', description: '管理组件', priority: 90 },
        { text: 'override', type: 'subcommand', description: '目录覆盖 toolchain', priority: 88 },
        { text: 'run', type: 'subcommand', description: '在指定 toolchain 下运行命令', priority: 86, usage: 'rustup run stable cargo --version' },
        { text: 'which', type: 'subcommand', description: '定位可执行文件', priority: 84, usage: 'rustup which cargo' },
        { text: 'doc', type: 'subcommand', description: '打开文档', priority: 82, usage: 'rustup doc --book' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        { text: '--version', type: 'option', description: '显示版本', priority: 48 },
    ],
    subcommands: {
        show: { name: 'show', description: '显示配置', options: [] },
        update: { name: 'update', description: '更新 toolchain', options: [] },
        default: { name: 'default', description: '设置默认 toolchain', options: [] },
        toolchain: rustupToolchain,
        target: rustupTarget,
        component: rustupComponent,
        override: rustupOverride,
        run: { name: 'run', description: '运行命令', options: [] },
        which: { name: 'which', description: '定位可执行文件', options: [] },
        doc: { name: 'doc', description: '打开文档', options: [] },
    },
};

export default rustupCommand;


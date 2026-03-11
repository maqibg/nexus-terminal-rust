
import type { CommandDefinition } from '../types';

/**
 * go 命令定义
 */

const goMod: CommandDefinition = {
    name: 'mod',
    description: 'Go Modules 管理',
    options: [
        { text: 'init', type: 'subcommand', description: '初始化模块', priority: 100, usage: 'go mod init example.com/app' },
        { text: 'tidy', type: 'subcommand', description: '整理依赖', priority: 95, usage: 'go mod tidy' },
        { text: 'download', type: 'subcommand', description: '下载依赖', priority: 90, usage: 'go mod download' },
        { text: 'vendor', type: 'subcommand', description: '生成 vendor 目录', priority: 85, usage: 'go mod vendor' },
        { text: 'graph', type: 'subcommand', description: '依赖图', priority: 80 },
        { text: 'why', type: 'subcommand', description: '解释为何依赖', priority: 75, usage: 'go mod why -m pkg' },
    ],
    subcommands: {
        init: { name: 'init', description: '初始化模块', options: [] },
        tidy: { name: 'tidy', description: '整理依赖', options: [] },
        download: { name: 'download', description: '下载依赖', options: [] },
        vendor: { name: 'vendor', description: '生成 vendor 目录', options: [] },
        graph: { name: 'graph', description: '依赖图', options: [] },
        why: { name: 'why', description: '解释依赖', options: [] },
    },
};

const goTool: CommandDefinition = {
    name: 'tool',
    description: '运行 go 工具链工具',
    options: [
        { text: 'pprof', type: 'subcommand', description: '性能分析', priority: 100, usage: 'go tool pprof' },
        { text: 'cover', type: 'subcommand', description: '覆盖率工具', priority: 95, usage: 'go tool cover' },
        { text: 'compile', type: 'subcommand', description: '编译器（底层）', priority: 90, usage: 'go tool compile' },
        { text: 'link', type: 'subcommand', description: '链接器（底层）', priority: 85, usage: 'go tool link' },
    ],
    subcommands: {
        pprof: { name: 'pprof', description: '性能分析', options: [] },
        cover: { name: 'cover', description: '覆盖率工具', options: [] },
        compile: { name: 'compile', description: '编译器', options: [] },
        link: { name: 'link', description: '链接器', options: [] },
    },
};

const goCommand: CommandDefinition = {
    name: 'go',
    description: 'Go 工具链',
    options: [
        { text: 'build', type: 'subcommand', description: '构建包', priority: 100, usage: 'go build ./...' },
        { text: 'run', type: 'subcommand', description: '运行程序', priority: 98, usage: 'go run .' },
        { text: 'test', type: 'subcommand', description: '运行测试', priority: 96, usage: 'go test ./...' },
        { text: 'fmt', type: 'subcommand', description: '格式化代码', priority: 94, usage: 'go fmt ./...' },
        { text: 'vet', type: 'subcommand', description: '静态检查', priority: 92, usage: 'go vet ./...' },
        { text: 'mod', type: 'subcommand', description: 'Go Modules', priority: 90 },
        { text: 'get', type: 'subcommand', description: '获取依赖（旧用法）', priority: 88, usage: 'go get pkg' },
        { text: 'install', type: 'subcommand', description: '安装包/命令', priority: 86, usage: 'go install pkg@latest' },
        { text: 'list', type: 'subcommand', description: '列出包信息', priority: 84, usage: 'go list ./...' },
        { text: 'env', type: 'subcommand', description: '查看/设置环境', priority: 82, usage: 'go env' },
        { text: 'clean', type: 'subcommand', description: '清理缓存/产物', priority: 80, usage: 'go clean -cache' },
        { text: 'generate', type: 'subcommand', description: '代码生成', priority: 78, usage: 'go generate ./...' },
        { text: 'doc', type: 'subcommand', description: '文档', priority: 76, usage: 'go doc fmt' },
        { text: 'tool', type: 'subcommand', description: '工具', priority: 74 },
        { text: 'version', type: 'subcommand', description: '版本', priority: 72, usage: 'go version' },
        { text: 'help', type: 'subcommand', description: '帮助', priority: 50, usage: 'go help test' },
    ],
    subcommands: {
        build: { name: 'build', description: '构建', options: [] },
        run: { name: 'run', description: '运行', options: [] },
        test: { name: 'test', description: '测试', options: [] },
        fmt: { name: 'fmt', description: '格式化', options: [] },
        vet: { name: 'vet', description: '静态检查', options: [] },
        mod: goMod,
        get: { name: 'get', description: '获取依赖', options: [] },
        install: { name: 'install', description: '安装', options: [] },
        list: { name: 'list', description: '列出包信息', options: [] },
        env: { name: 'env', description: '环境变量', options: [] },
        clean: { name: 'clean', description: '清理', options: [] },
        generate: { name: 'generate', description: '代码生成', options: [] },
        doc: { name: 'doc', description: '文档', options: [] },
        tool: goTool,
        version: { name: 'version', description: '版本', options: [] },
        help: { name: 'help', description: '帮助', options: [] },
    },
};

export default goCommand;


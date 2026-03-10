
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * gcloud 命令定义（常用子命令集合）
 */

const GCLOUD_FORMAT_HINTS: CompletionItem[] = [
    { text: 'json', type: 'subcommand', description: 'JSON 输出', priority: 90, matchPart: '', restPart: 'json' },
    { text: 'yaml', type: 'subcommand', description: 'YAML 输出', priority: 85, matchPart: '', restPart: 'yaml' },
    { text: 'table', type: 'subcommand', description: '表格输出', priority: 80, matchPart: '', restPart: 'table' },
    { text: 'value(name)', type: 'subcommand', description: '只输出字段', priority: 75, matchPart: '', restPart: 'value(name)' },
];

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

const gcloudAuth: CommandDefinition = {
    name: 'auth',
    description: '认证',
    options: [
        { text: 'login', type: 'subcommand', description: '登录', priority: 100 },
        { text: 'list', type: 'subcommand', description: '列出账号', priority: 90 },
        { text: 'activate-service-account', type: 'subcommand', description: '激活服务账号', priority: 85 },
        { text: 'revoke', type: 'subcommand', description: '撤销授权', priority: 80 },
    ],
};

const gcloudConfig: CommandDefinition = {
    name: 'config',
    description: '配置',
    options: [
        { text: 'list', type: 'subcommand', description: '列出配置', priority: 95 },
        { text: 'set', type: 'subcommand', description: '设置配置', priority: 90, usage: 'gcloud config set project PROJECT_ID' },
        { text: 'get-value', type: 'subcommand', description: '读取配置值', priority: 85 },
    ],
};

const gcloudCompute: CommandDefinition = {
    name: 'compute',
    description: 'Compute Engine',
    options: [
        { text: 'instances', type: 'subcommand', description: '实例', priority: 100 },
        { text: 'ssh', type: 'subcommand', description: 'SSH 到实例', priority: 95 },
        { text: 'scp', type: 'subcommand', description: 'SCP 传输', priority: 90 },
        { text: 'disks', type: 'subcommand', description: '磁盘', priority: 85 },
        { text: 'networks', type: 'subcommand', description: '网络', priority: 80 },
        { text: 'firewall-rules', type: 'subcommand', description: '防火墙规则', priority: 80 },
    ],
};

const gcloudContainer: CommandDefinition = {
    name: 'container',
    description: 'GKE',
    options: [
        { text: 'clusters', type: 'subcommand', description: '集群', priority: 100 },
        { text: 'get-credentials', type: 'subcommand', description: '获取凭据', priority: 90, usage: 'gcloud container clusters get-credentials NAME' },
    ],
};

const gcloudRun: CommandDefinition = {
    name: 'run',
    description: 'Cloud Run',
    options: [
        { text: 'deploy', type: 'subcommand', description: '部署服务', priority: 100 },
        { text: 'services', type: 'subcommand', description: '服务管理', priority: 90 },
        { text: 'logs', type: 'subcommand', description: '日志', priority: 85 },
    ],
};

const gcloudStorage: CommandDefinition = {
    name: 'storage',
    description: 'Cloud Storage',
    options: [
        { text: 'ls', type: 'subcommand', description: '列出 bucket/对象', priority: 100 },
        { text: 'cp', type: 'subcommand', description: '复制对象', priority: 95 },
        { text: 'rm', type: 'subcommand', description: '删除对象', priority: 90 },
        { text: 'rsync', type: 'subcommand', description: '同步', priority: 85 },
    ],
};

const gcloudCommand: CommandDefinition = {
    name: 'gcloud',
    description: 'Google Cloud CLI',
    options: [
        // global
        { text: '--project', type: 'option', description: '指定项目', priority: 95, usage: '--project PROJECT_ID' },
        { text: '--account', type: 'option', description: '指定账号', priority: 90, usage: '--account user@example.com' },
        { text: '--configuration', type: 'option', description: '指定配置', priority: 85 },
        { text: '--quiet', type: 'option', description: '静默模式', priority: 80 },
        { text: '--format', type: 'option', description: '输出格式', priority: 75, usage: '--format json' },
        { text: '--verbosity', type: 'option', description: '日志级别', priority: 70, usage: '--verbosity debug' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        // subcommands
        { text: 'auth', type: 'subcommand', description: '认证', priority: 100 },
        { text: 'config', type: 'subcommand', description: '配置', priority: 95 },
        { text: 'compute', type: 'subcommand', description: 'Compute Engine', priority: 90 },
        { text: 'container', type: 'subcommand', description: 'GKE', priority: 85 },
        { text: 'run', type: 'subcommand', description: 'Cloud Run', priority: 80 },
        { text: 'storage', type: 'subcommand', description: 'Cloud Storage', priority: 80 },
        { text: 'projects', type: 'subcommand', description: '项目', priority: 75 },
        { text: 'iam', type: 'subcommand', description: 'IAM', priority: 70 },
    ],
    subcommands: {
        'auth': gcloudAuth,
        'config': gcloudConfig,
        'compute': gcloudCompute,
        'container': gcloudContainer,
        'run': gcloudRun,
        'storage': gcloudStorage,
    },
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '--format') {
            return filterHints(GCLOUD_FORMAT_HINTS, ctx.currentArg).map((item) => ({
                ...item,
                matchPart: ctx.currentArg,
                restPart: item.text.substring(ctx.currentArg.length),
            }));
        }
        return [];
    },
};

export default gcloudCommand;


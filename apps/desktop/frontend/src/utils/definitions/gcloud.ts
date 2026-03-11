
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
    subcommands: {
        login: { name: 'login', description: '登录', options: [] },
        list: { name: 'list', description: '列出账号', options: [] },
        'activate-service-account': { name: 'activate-service-account', description: '激活服务账号', options: [] },
        revoke: { name: 'revoke', description: '撤销授权', options: [] },
    },
};

const gcloudConfig: CommandDefinition = {
    name: 'config',
    description: '配置',
    options: [
        { text: 'list', type: 'subcommand', description: '列出配置', priority: 95 },
        { text: 'set', type: 'subcommand', description: '设置配置', priority: 90, usage: 'gcloud config set project PROJECT_ID' },
        { text: 'get-value', type: 'subcommand', description: '读取配置值', priority: 85 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出配置', options: [] },
        set: { name: 'set', description: '设置配置', options: [] },
        'get-value': { name: 'get-value', description: '读取配置值', options: [] },
    },
};

const gcloudComputeInstances: CommandDefinition = {
    name: 'instances',
    description: '实例',
    options: [
        { text: 'list', type: 'subcommand', description: '列出实例', priority: 100, usage: 'gcloud compute instances list' },
        { text: 'describe', type: 'subcommand', description: '查看详情', priority: 90, usage: 'gcloud compute instances describe NAME' },
        { text: 'create', type: 'subcommand', description: '创建实例', priority: 85 },
        { text: 'delete', type: 'subcommand', description: '删除实例', priority: 80 },
        { text: 'start', type: 'subcommand', description: '启动实例', priority: 75 },
        { text: 'stop', type: 'subcommand', description: '停止实例', priority: 75 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出实例', options: [] },
        describe: { name: 'describe', description: '查看详情', options: [] },
        create: { name: 'create', description: '创建实例', options: [] },
        delete: { name: 'delete', description: '删除实例', options: [] },
        start: { name: 'start', description: '启动实例', options: [] },
        stop: { name: 'stop', description: '停止实例', options: [] },
    },
};

const gcloudComputeDisks: CommandDefinition = {
    name: 'disks',
    description: '磁盘',
    options: [
        { text: 'list', type: 'subcommand', description: '列出磁盘', priority: 95 },
        { text: 'describe', type: 'subcommand', description: '查看详情', priority: 85 },
        { text: 'create', type: 'subcommand', description: '创建磁盘', priority: 80 },
        { text: 'delete', type: 'subcommand', description: '删除磁盘', priority: 75 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出磁盘', options: [] },
        describe: { name: 'describe', description: '查看详情', options: [] },
        create: { name: 'create', description: '创建磁盘', options: [] },
        delete: { name: 'delete', description: '删除磁盘', options: [] },
    },
};

const gcloudComputeNetworks: CommandDefinition = {
    name: 'networks',
    description: '网络',
    options: [
        { text: 'list', type: 'subcommand', description: '列出网络', priority: 95 },
        { text: 'describe', type: 'subcommand', description: '查看详情', priority: 85 },
        { text: 'create', type: 'subcommand', description: '创建网络', priority: 80 },
        { text: 'delete', type: 'subcommand', description: '删除网络', priority: 75 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出网络', options: [] },
        describe: { name: 'describe', description: '查看详情', options: [] },
        create: { name: 'create', description: '创建网络', options: [] },
        delete: { name: 'delete', description: '删除网络', options: [] },
    },
};

const gcloudComputeFirewallRules: CommandDefinition = {
    name: 'firewall-rules',
    description: '防火墙规则',
    options: [
        { text: 'list', type: 'subcommand', description: '列出规则', priority: 95 },
        { text: 'describe', type: 'subcommand', description: '查看详情', priority: 85 },
        { text: 'create', type: 'subcommand', description: '创建规则', priority: 80 },
        { text: 'delete', type: 'subcommand', description: '删除规则', priority: 75 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出规则', options: [] },
        describe: { name: 'describe', description: '查看详情', options: [] },
        create: { name: 'create', description: '创建规则', options: [] },
        delete: { name: 'delete', description: '删除规则', options: [] },
    },
};

const gcloudComputeSsh: CommandDefinition = {
    name: 'ssh',
    description: 'SSH 到实例',
    options: [
        { text: '--zone', type: 'option', description: '指定 zone', priority: 85, usage: '--zone asia-east1-a' },
        { text: '--tunnel-through-iap', type: 'option', description: '通过 IAP 隧道', priority: 75 },
        { text: '--command', type: 'option', description: '远端命令', priority: 70, usage: '--command \"uname -a\"' },
    ],
};

const gcloudComputeScp: CommandDefinition = {
    name: 'scp',
    description: 'SCP 传输',
    options: [
        { text: '--zone', type: 'option', description: '指定 zone', priority: 85 },
        { text: '--recurse', type: 'option', description: '递归复制', priority: 80 },
        { text: '--compress', type: 'option', description: '启用压缩', priority: 75 },
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
    subcommands: {
        instances: gcloudComputeInstances,
        ssh: gcloudComputeSsh,
        scp: gcloudComputeScp,
        disks: gcloudComputeDisks,
        networks: gcloudComputeNetworks,
        'firewall-rules': gcloudComputeFirewallRules,
    },
};

const gcloudContainer: CommandDefinition = {
    name: 'container',
    description: 'GKE',
    options: [
        { text: 'clusters', type: 'subcommand', description: '集群', priority: 100 },
        { text: 'get-credentials', type: 'subcommand', description: '获取凭据', priority: 90, usage: 'gcloud container clusters get-credentials NAME' },
    ],
    subcommands: {
        clusters: {
            name: 'clusters',
            description: '集群',
            options: [
                { text: 'list', type: 'subcommand', description: '列出集群', priority: 95 },
                { text: 'describe', type: 'subcommand', description: '查看详情', priority: 85 },
                { text: 'get-credentials', type: 'subcommand', description: '获取凭据', priority: 80 },
                { text: 'create', type: 'subcommand', description: '创建集群', priority: 75 },
                { text: 'delete', type: 'subcommand', description: '删除集群', priority: 70 },
            ],
            subcommands: {
                list: { name: 'list', description: '列出集群', options: [] },
                describe: { name: 'describe', description: '查看详情', options: [] },
                'get-credentials': { name: 'get-credentials', description: '获取凭据', options: [] },
                create: { name: 'create', description: '创建集群', options: [] },
                delete: { name: 'delete', description: '删除集群', options: [] },
            },
        },
        'get-credentials': {
            name: 'get-credentials',
            description: '获取凭据',
            options: [
                { text: '--zone', type: 'option', description: '指定 zone', priority: 85 },
                { text: '--region', type: 'option', description: '指定 region', priority: 80 },
                { text: '--project', type: 'option', description: '指定项目', priority: 75 },
            ],
        },
    },
};

const gcloudRun: CommandDefinition = {
    name: 'run',
    description: 'Cloud Run',
    options: [
        { text: 'deploy', type: 'subcommand', description: '部署服务', priority: 100 },
        { text: 'services', type: 'subcommand', description: '服务管理', priority: 90 },
        { text: 'logs', type: 'subcommand', description: '日志', priority: 85 },
    ],
    subcommands: {
        deploy: {
            name: 'deploy',
            description: '部署服务',
            options: [
                { text: '--image', type: 'option', description: '镜像', priority: 90, usage: '--image gcr.io/PROJECT/IMAGE' },
                { text: '--region', type: 'option', description: '区域', priority: 85 },
                { text: '--platform', type: 'option', description: '平台', priority: 80, usage: '--platform managed' },
                { text: '--allow-unauthenticated', type: 'option', description: '允许匿名访问', priority: 75 },
            ],
        },
        services: {
            name: 'services',
            description: '服务管理',
            options: [
                { text: 'list', type: 'subcommand', description: '列出服务', priority: 95 },
                { text: 'describe', type: 'subcommand', description: '查看详情', priority: 85 },
                { text: 'delete', type: 'subcommand', description: '删除服务', priority: 75 },
            ],
            subcommands: {
                list: { name: 'list', description: '列出服务', options: [] },
                describe: { name: 'describe', description: '查看详情', options: [] },
                delete: { name: 'delete', description: '删除服务', options: [] },
            },
        },
        logs: {
            name: 'logs',
            description: '日志',
            options: [
                { text: 'read', type: 'subcommand', description: '读取日志', priority: 90, usage: 'gcloud run logs read' },
            ],
            subcommands: {
                read: { name: 'read', description: '读取日志', options: [] },
            },
        },
    },
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
    subcommands: {
        ls: {
            name: 'ls',
            description: '列出 bucket/对象',
            options: [
                { text: '--recursive', type: 'option', description: '递归列出', priority: 80 },
            ],
        },
        cp: {
            name: 'cp',
            description: '复制对象',
            options: [
                { text: '--recursive', type: 'option', description: '递归复制', priority: 85 },
                { text: '--preserve-posix', type: 'option', description: '保留权限/时间戳', priority: 70 },
            ],
        },
        rm: {
            name: 'rm',
            description: '删除对象',
            options: [
                { text: '--recursive', type: 'option', description: '递归删除', priority: 85 },
            ],
        },
        rsync: {
            name: 'rsync',
            description: '同步',
            options: [
                { text: '--recursive', type: 'option', description: '递归同步', priority: 85 },
                { text: '--delete-unmatched-destination-objects', type: 'option', description: '删除目标多余对象', priority: 75 },
            ],
        },
    },
};

const gcloudProjects: CommandDefinition = {
    name: 'projects',
    description: '项目',
    options: [
        { text: 'list', type: 'subcommand', description: '列出项目', priority: 95 },
        { text: 'describe', type: 'subcommand', description: '查看详情', priority: 85 },
    ],
    subcommands: {
        list: { name: 'list', description: '列出项目', options: [] },
        describe: { name: 'describe', description: '查看详情', options: [] },
    },
};

const gcloudIam: CommandDefinition = {
    name: 'iam',
    description: 'IAM',
    options: [
        { text: 'service-accounts', type: 'subcommand', description: '服务账号', priority: 90 },
        { text: 'roles', type: 'subcommand', description: '角色', priority: 80 },
    ],
    subcommands: {
        'service-accounts': {
            name: 'service-accounts',
            description: '服务账号',
            options: [
                { text: 'list', type: 'subcommand', description: '列出服务账号', priority: 95 },
                { text: 'describe', type: 'subcommand', description: '查看详情', priority: 85 },
            ],
            subcommands: {
                list: { name: 'list', description: '列出服务账号', options: [] },
                describe: { name: 'describe', description: '查看详情', options: [] },
            },
        },
        roles: {
            name: 'roles',
            description: '角色',
            options: [
                { text: 'list', type: 'subcommand', description: '列出角色', priority: 90 },
                { text: 'describe', type: 'subcommand', description: '查看详情', priority: 80 },
            ],
            subcommands: {
                list: { name: 'list', description: '列出角色', options: [] },
                describe: { name: 'describe', description: '查看详情', options: [] },
            },
        },
    },
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
        'projects': gcloudProjects,
        'iam': gcloudIam,
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

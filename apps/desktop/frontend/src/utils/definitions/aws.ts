
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * AWS CLI 命令定义
 * 目标：覆盖常用 service + 常用全局 flags
 */

const AWS_OUTPUT_HINTS: CompletionItem[] = [
    { text: 'json', type: 'subcommand', description: 'JSON 输出', priority: 90, matchPart: '', restPart: 'json' },
    { text: 'text', type: 'subcommand', description: '纯文本输出', priority: 85, matchPart: '', restPart: 'text' },
    { text: 'table', type: 'subcommand', description: '表格输出', priority: 80, matchPart: '', restPart: 'table' },
];

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

function shouldSuggestLocalPath(arg: string): boolean {
    if (!arg || arg.startsWith('-')) return false;
    if (arg.startsWith('s3://')) return false;
    if (arg.includes('://')) return false;
    return true;
}

const awsS3Cp: CommandDefinition = {
    name: 'cp',
    description: '复制对象',
    options: [
        { text: '--recursive', type: 'option', description: '递归复制', priority: 90 },
        { text: '--exclude', type: 'option', description: '排除模式', priority: 80, usage: '--exclude node_modules/*' },
        { text: '--include', type: 'option', description: '包含模式', priority: 75 },
        { text: '--dryrun', type: 'option', description: '演练，不实际执行', priority: 70 },
        { text: '--acl', type: 'option', description: 'ACL', priority: 65, usage: '--acl public-read' },
        { text: '--storage-class', type: 'option', description: '存储类型', priority: 60, usage: '--storage-class STANDARD_IA' },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        if (!shouldSuggestLocalPath(ctx.currentArg)) return [];
        return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
    },
};

const awsS3Sync: CommandDefinition = {
    name: 'sync',
    description: '同步目录',
    options: [
        { text: '--delete', type: 'option', description: '删除目标多余对象', priority: 85 },
        { text: '--exclude', type: 'option', description: '排除模式', priority: 80 },
        { text: '--include', type: 'option', description: '包含模式', priority: 75 },
        { text: '--dryrun', type: 'option', description: '演练，不实际执行', priority: 70 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        if (!shouldSuggestLocalPath(ctx.currentArg)) return [];
        return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
    },
};

const awsS3: CommandDefinition = {
    name: 's3',
    description: 'S3 高层命令',
    options: [
        { text: 'ls', type: 'subcommand', description: '列出 bucket/对象', priority: 100, usage: 'aws s3 ls' },
        { text: 'cp', type: 'subcommand', description: '复制对象', priority: 95, usage: 'aws s3 cp file s3://bucket/key' },
        { text: 'sync', type: 'subcommand', description: '同步目录', priority: 90, usage: 'aws s3 sync ./dir s3://bucket/prefix/' },
        { text: 'mb', type: 'subcommand', description: '创建 bucket', priority: 85 },
        { text: 'rb', type: 'subcommand', description: '删除 bucket', priority: 80 },
        { text: 'rm', type: 'subcommand', description: '删除对象', priority: 75 },
        { text: 'presign', type: 'subcommand', description: '生成预签名 URL', priority: 70 },
    ],
    subcommands: {
        ls: { name: 'ls', description: '列出 bucket/对象', options: [] },
        cp: awsS3Cp,
        sync: awsS3Sync,
        mb: { name: 'mb', description: '创建 bucket', options: [] },
        rb: { name: 'rb', description: '删除 bucket', options: [] },
        rm: { name: 'rm', description: '删除对象', options: [] },
        presign: { name: 'presign', description: '生成预签名 URL', options: [] },
    },
};

const awsSts: CommandDefinition = {
    name: 'sts',
    description: 'STS',
    options: [
        { text: 'get-caller-identity', type: 'subcommand', description: '当前身份', priority: 100 },
        { text: 'assume-role', type: 'subcommand', description: '扮演角色', priority: 90, usage: 'aws sts assume-role --role-arn ...' },
    ],
    subcommands: {
        'get-caller-identity': { name: 'get-caller-identity', description: '当前身份', options: [] },
        'assume-role': { name: 'assume-role', description: '扮演角色', options: [] },
    },
};

const awsEcr: CommandDefinition = {
    name: 'ecr',
    description: 'ECR',
    options: [
        { text: 'get-login-password', type: 'subcommand', description: '获取登录密码', priority: 100 },
        { text: 'describe-repositories', type: 'subcommand', description: '列出仓库', priority: 90 },
    ],
    subcommands: {
        'get-login-password': { name: 'get-login-password', description: '获取登录密码', options: [] },
        'describe-repositories': { name: 'describe-repositories', description: '列出仓库', options: [] },
    },
};

const awsLogs: CommandDefinition = {
    name: 'logs',
    description: 'CloudWatch Logs',
    options: [
        { text: 'describe-log-groups', type: 'subcommand', description: '列出日志组', priority: 95 },
        { text: 'describe-log-streams', type: 'subcommand', description: '列出日志流', priority: 90 },
        { text: 'tail', type: 'subcommand', description: '实时追踪日志', priority: 85, usage: 'aws logs tail /aws/lambda/name --follow' },
    ],
    subcommands: {
        'describe-log-groups': { name: 'describe-log-groups', description: '列出日志组', options: [] },
        'describe-log-streams': { name: 'describe-log-streams', description: '列出日志流', options: [] },
        tail: { name: 'tail', description: '实时追踪日志', options: [] },
    },
};

const awsS3Api: CommandDefinition = {
    name: 's3api',
    description: 'S3 API 级命令',
    options: [],
};

const awsEc2: CommandDefinition = {
    name: 'ec2',
    description: 'EC2',
    options: [],
};

const awsIam: CommandDefinition = {
    name: 'iam',
    description: 'IAM',
    options: [],
};

const awsLambda: CommandDefinition = {
    name: 'lambda',
    description: 'Lambda',
    options: [],
};

const awsConfigure: CommandDefinition = {
    name: 'configure',
    description: '配置',
    options: [
        { text: 'list', type: 'subcommand', description: '列出配置', priority: 90 },
        { text: 'get', type: 'subcommand', description: '读取配置项', priority: 80, usage: 'aws configure get region' },
        { text: 'set', type: 'subcommand', description: '设置配置项', priority: 75, usage: 'aws configure set region ap-northeast-1' },
    ],
    subcommands: {
        list: { name: 'list', description: '列出配置', options: [] },
        get: { name: 'get', description: '读取配置项', options: [] },
        set: { name: 'set', description: '设置配置项', options: [] },
    },
};

const awsCommand: CommandDefinition = {
    name: 'aws',
    description: 'AWS CLI',
    options: [
        // global
        { text: '--profile', type: 'option', description: '指定 profile', priority: 95, usage: '--profile default' },
        { text: '--region', type: 'option', description: '指定 region', priority: 95, usage: '--region ap-northeast-1' },
        { text: '--output', type: 'option', description: '输出格式', priority: 90, usage: '--output json' },
        { text: '--query', type: 'option', description: 'JMESPath 查询', priority: 80, usage: '--query "Reservations[*].Instances[*].InstanceId"' },
        { text: '--endpoint-url', type: 'option', description: '自定义 endpoint', priority: 75 },
        { text: '--no-paginate', type: 'option', description: '禁用分页器', priority: 70 },
        { text: '--cli-connect-timeout', type: 'option', description: '连接超时(秒)', priority: 65 },
        { text: '--cli-read-timeout', type: 'option', description: '读取超时(秒)', priority: 65 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        // services
        { text: 's3', type: 'subcommand', description: 'S3', priority: 100 },
        { text: 's3api', type: 'subcommand', description: 'S3 API 级命令', priority: 90 },
        { text: 'ec2', type: 'subcommand', description: 'EC2', priority: 90 },
        { text: 'iam', type: 'subcommand', description: 'IAM', priority: 85 },
        { text: 'sts', type: 'subcommand', description: 'STS', priority: 85 },
        { text: 'ecr', type: 'subcommand', description: 'ECR', priority: 80 },
        { text: 'lambda', type: 'subcommand', description: 'Lambda', priority: 75 },
        { text: 'logs', type: 'subcommand', description: 'CloudWatch Logs', priority: 75 },
        { text: 'configure', type: 'subcommand', description: '配置', priority: 70, usage: 'aws configure' },
    ],
    subcommands: {
        's3': awsS3,
        's3api': awsS3Api,
        'ec2': awsEc2,
        'iam': awsIam,
        'sts': awsSts,
        'ecr': awsEcr,
        'lambda': awsLambda,
        'logs': awsLogs,
        'configure': awsConfigure,
    },
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        // 给 --output 提供 hint；给 s3 cp/sync 提供路径 hint（简化版）
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '--output') {
            return filterHints(AWS_OUTPUT_HINTS, ctx.currentArg).map((item) => ({
                ...item,
                matchPart: ctx.currentArg,
                restPart: item.text.substring(ctx.currentArg.length),
            }));
        }

        if (!ctx.currentArg.startsWith('-')) {
            const sub = ctx.args[1] ?? '';
            if (sub === 's3' && (ctx.args.includes('cp') || ctx.args.includes('sync'))) {
                // 本地/远端路径都可能出现，这里只对普通路径给远端文件提示
                return getRemoteFiles(ctx.sessionId, ctx.currentArg, ctx.electronAPI);
            }
        }

        return [];
    },
};

export default awsCommand;

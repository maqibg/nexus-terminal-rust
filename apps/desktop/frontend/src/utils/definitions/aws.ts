
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
};

const awsSts: CommandDefinition = {
    name: 'sts',
    description: 'STS',
    options: [
        { text: 'get-caller-identity', type: 'subcommand', description: '当前身份', priority: 100 },
        { text: 'assume-role', type: 'subcommand', description: '扮演角色', priority: 90, usage: 'aws sts assume-role --role-arn ...' },
    ],
};

const awsEcr: CommandDefinition = {
    name: 'ecr',
    description: 'ECR',
    options: [
        { text: 'get-login-password', type: 'subcommand', description: '获取登录密码', priority: 100 },
        { text: 'describe-repositories', type: 'subcommand', description: '列出仓库', priority: 90 },
    ],
};

const awsLogs: CommandDefinition = {
    name: 'logs',
    description: 'CloudWatch Logs',
    options: [
        { text: 'describe-log-groups', type: 'subcommand', description: '列出日志组', priority: 95 },
        { text: 'describe-log-streams', type: 'subcommand', description: '列出日志流', priority: 90 },
        { text: 'tail', type: 'subcommand', description: '实时追踪日志', priority: 85, usage: 'aws logs tail /aws/lambda/name --follow' },
    ],
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
        'sts': awsSts,
        'ecr': awsEcr,
        'logs': awsLogs,
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


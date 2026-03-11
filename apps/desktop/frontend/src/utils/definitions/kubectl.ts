
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles, getRemotePathSuggestions } from '../providers/file-system';

/**
 * kubectl 命令定义（常用子命令 + 常用全局参数）
 */

const RESOURCE_TYPES: CompletionItem[] = [
    { text: 'pods', type: 'subcommand', description: 'Pod', priority: 95, matchPart: '', restPart: 'pods' },
    { text: 'deployments', type: 'subcommand', description: 'Deployment', priority: 90, matchPart: '', restPart: 'deployments' },
    { text: 'services', type: 'subcommand', description: 'Service', priority: 88, matchPart: '', restPart: 'services' },
    { text: 'nodes', type: 'subcommand', description: 'Node', priority: 86, matchPart: '', restPart: 'nodes' },
    { text: 'namespaces', type: 'subcommand', description: 'Namespace', priority: 84, matchPart: '', restPart: 'namespaces' },
    { text: 'configmaps', type: 'subcommand', description: 'ConfigMap', priority: 82, matchPart: '', restPart: 'configmaps' },
    { text: 'secrets', type: 'subcommand', description: 'Secret', priority: 80, matchPart: '', restPart: 'secrets' },
    { text: 'ingresses', type: 'subcommand', description: 'Ingress', priority: 78, matchPart: '', restPart: 'ingresses' },
    { text: 'jobs', type: 'subcommand', description: 'Job', priority: 76, matchPart: '', restPart: 'jobs' },
    { text: 'cronjobs', type: 'subcommand', description: 'CronJob', priority: 74, matchPart: '', restPart: 'cronjobs' },
    { text: 'statefulsets', type: 'subcommand', description: 'StatefulSet', priority: 72, matchPart: '', restPart: 'statefulsets' },
    { text: 'daemonsets', type: 'subcommand', description: 'DaemonSet', priority: 70, matchPart: '', restPart: 'daemonsets' },
    { text: 'pvc', type: 'subcommand', description: 'PersistentVolumeClaim', priority: 68, matchPart: '', restPart: 'pvc' },
    { text: 'pv', type: 'subcommand', description: 'PersistentVolume', priority: 67, matchPart: '', restPart: 'pv' },
];

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

async function executeKubectl(ctx: CompletionContext, command: string, timeout = 1600): Promise<string | null> {
    if (!ctx.sessionId || !ctx.electronAPI) return null;
    try {
        const result = await ctx.electronAPI.ssh?.executeCommand?.(ctx.sessionId, command, timeout);
        if (result?.success && result.data) {
            return String(result.data).trim();
        }
    } catch {
        // ignore
    }
    return null;
}

let nsCache: { value: CompletionItem[]; ts: number } | null = null;
async function getNamespaces(ctx: CompletionContext): Promise<CompletionItem[]> {
    const now = Date.now();
    if (nsCache && now - nsCache.ts < 5000) return nsCache.value;

    const out = await executeKubectl(ctx, `kubectl get ns -o name 2>/dev/null | head -50`);
    if (!out) return [];
    const items = out.split('\n')
        .map((line) => line.trim())
        .filter(Boolean)
        .map((line) => line.includes('/') ? line.split('/').pop() : line)
        .filter((v): v is string => Boolean(v));

    const value = items.map((name) => ({
        text: name,
        type: 'subcommand' as const,
        description: 'namespace',
        priority: 90,
        matchPart: '',
        restPart: name,
    }));
    nsCache = { value, ts: now };
    return value;
}

let ctxCache: { value: CompletionItem[]; ts: number } | null = null;
async function getContexts(ctx: CompletionContext): Promise<CompletionItem[]> {
    const now = Date.now();
    if (ctxCache && now - ctxCache.ts < 5000) return ctxCache.value;

    const out = await executeKubectl(ctx, `kubectl config get-contexts -o name 2>/dev/null | head -80`);
    if (!out) return [];
    const items = out.split('\n').map((v) => v.trim()).filter(Boolean);
    const value = items.map((name) => ({
        text: name,
        type: 'subcommand' as const,
        description: 'context',
        priority: 85,
        matchPart: '',
        restPart: name,
    }));
    ctxCache = { value, ts: now };
    return value;
}

const kubectlCommand: CommandDefinition = {
    name: 'kubectl',
    description: 'Kubernetes CLI',
    options: [
        // global flags
        { text: '-n', type: 'option', description: 'namespace', priority: 100, usage: '-n default' },
        { text: '--namespace', type: 'option', description: 'namespace', priority: 98 },
        { text: '-A', type: 'option', description: '所有 namespace', priority: 95 },
        { text: '--all-namespaces', type: 'option', description: '所有 namespace', priority: 95 },
        { text: '--context', type: 'option', description: '指定 context', priority: 92 },
        { text: '--kubeconfig', type: 'option', description: 'kubeconfig 路径', priority: 90 },
        { text: '-o', type: 'option', description: '输出格式', priority: 88, usage: '-o wide' },
        { text: '--output', type: 'option', description: '输出格式', priority: 88 },
        { text: '-l', type: 'option', description: 'label selector', priority: 86, usage: '-l app=web' },
        { text: '--selector', type: 'option', description: 'label selector', priority: 86 },
        { text: '-f', type: 'option', description: '文件/目录', priority: 85, usage: '-f ./deploy.yaml', repeatable: true },
        { text: '--filename', type: 'option', description: '文件/目录', priority: 85, repeatable: true },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
        // subcommands
        { text: 'get', type: 'subcommand', description: '获取资源', priority: 100, usage: 'kubectl get pods' },
        { text: 'describe', type: 'subcommand', description: '查看详情', priority: 95 },
        { text: 'logs', type: 'subcommand', description: '查看日志', priority: 92 },
        { text: 'exec', type: 'subcommand', description: '进入容器执行命令', priority: 90 },
        { text: 'apply', type: 'subcommand', description: '应用配置', priority: 90, usage: 'kubectl apply -f file.yaml' },
        { text: 'delete', type: 'subcommand', description: '删除资源', priority: 88 },
        { text: 'edit', type: 'subcommand', description: '编辑资源', priority: 80 },
        { text: 'config', type: 'subcommand', description: '配置', priority: 78 },
        { text: 'version', type: 'subcommand', description: '版本', priority: 70 },
    ],
    subcommands: {
        get: { name: 'get', description: '获取资源', options: [] },
        describe: { name: 'describe', description: '查看详情', options: [] },
        logs: { name: 'logs', description: '查看日志', options: [] },
        exec: { name: 'exec', description: '进入容器执行命令', options: [] },
        apply: { name: 'apply', description: '应用配置', options: [] },
        delete: { name: 'delete', description: '删除资源', options: [] },
        edit: { name: 'edit', description: '编辑资源', options: [] },
        config: { name: 'config', description: '配置', options: [] },
        version: { name: 'version', description: '版本', options: [] },
    },
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-n' || prevArg === '--namespace') {
            const items = await getNamespaces(ctx);
            return filterHints(items, ctx.currentArg);
        }
        if (prevArg === '--context') {
            const items = await getContexts(ctx);
            return filterHints(items, ctx.currentArg);
        }
        if (prevArg === '--kubeconfig') {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        if (prevArg === '-f' || prevArg === '--filename') {
            // 允许空前缀也给出路径建议
            const inputPath = ctx.currentArg || './';
            return getRemotePathSuggestions(ctx.sessionId, inputPath, { foldersOnly: false, electronAPI: ctx.electronAPI });
        }

        const sub = ctx.args[1] ?? '';
        if ((sub === 'get' || sub === 'describe' || sub === 'delete' || sub === 'edit') && ctx.currentArgIndex === 2) {
            return filterHints(RESOURCE_TYPES, ctx.currentArg).map((item) => ({
                ...item,
                matchPart: ctx.currentArg,
                restPart: item.text.substring(ctx.currentArg.length),
            }));
        }
        return [];
    },
};

export default kubectlCommand;

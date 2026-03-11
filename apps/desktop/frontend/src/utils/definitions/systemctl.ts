
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';

/**
 * 获取系统服务列表
 */
async function getSystemServices(ctx: CompletionContext): Promise<CompletionItem[]> {
    if (!ctx.sessionId || !ctx.electronAPI) return [];

    try {
        const result = await ctx.electronAPI.ssh?.executeCommand?.(
            ctx.sessionId,
            `systemctl list-units --type=service --no-pager --plain 2>/dev/null | head -30 | awk '{print $1}'`,
            5000
        );

        if (!result?.success || !result.data) return [];

        return result.data.split('\n')
            .filter((line: string) => line.trim() && line.endsWith('.service'))
            .map((service: string) => ({
                text: service.replace('.service', ''),
                displayText: service,
                type: 'hint' as const,
                description: '系统服务',
                priority: 80,
                matchPart: '',
                restPart: service.replace('.service', '')
            }));
    } catch {
        return [];
    }
}

// systemctl 子命令
const systemctlStart: CommandDefinition = {
    name: 'start',
    description: '启动服务',
    options: [],
    generate: getSystemServices
};

const systemctlStop: CommandDefinition = {
    name: 'stop',
    description: '停止服务',
    options: [],
    generate: getSystemServices
};

const systemctlRestart: CommandDefinition = {
    name: 'restart',
    description: '重启服务',
    options: [],
    generate: getSystemServices
};

const systemctlStatus: CommandDefinition = {
    name: 'status',
    description: '查看服务状态',
    options: [],
    generate: getSystemServices
};

const systemctlEnable: CommandDefinition = {
    name: 'enable',
    description: '设置开机启动',
    options: [
        { text: '--now', type: 'option', description: '同时启动', priority: 90 }
    ],
    generate: getSystemServices
};

const systemctlDisable: CommandDefinition = {
    name: 'disable',
    description: '取消开机启动',
    options: [
        { text: '--now', type: 'option', description: '同时停止', priority: 90 }
    ],
    generate: getSystemServices
};

const systemctlDaemonReload: CommandDefinition = {
    name: 'daemon-reload',
    description: '重载 daemon',
    options: [
        { text: '--user', type: 'option', description: '用户模式', priority: 70 },
        { text: '--system', type: 'option', description: '系统模式', priority: 70 },
    ],
};

const systemctlListUnits: CommandDefinition = {
    name: 'list-units',
    description: '列出 units',
    options: [
        { text: '--type=service', type: 'option', description: '仅 service', priority: 90 },
        { text: '--all', type: 'option', description: '包含未加载', priority: 85 },
        { text: '--state', type: 'option', description: '按状态过滤', priority: 80, usage: '--state running' },
        { text: '--no-pager', type: 'option', description: '不分页', priority: 70 },
    ],
};

/**
 * systemctl 命令 - 系统服务管理
 */
const systemctlCommand: CommandDefinition = {
    name: 'systemctl',
    description: '系统服务管理',
    options: [
        { text: 'status', type: 'subcommand', description: '查看服务状态', priority: 100, usage: 'systemctl status nginx' },
        { text: 'start', type: 'subcommand', description: '启动服务', priority: 95, usage: 'systemctl start nginx' },
        { text: 'stop', type: 'subcommand', description: '停止服务', priority: 95, usage: 'systemctl stop nginx' },
        { text: 'restart', type: 'subcommand', description: '重启服务', priority: 90, usage: 'systemctl restart nginx' },
        { text: 'reload', type: 'subcommand', description: '重载配置', priority: 85, usage: 'systemctl reload nginx' },
        { text: 'enable', type: 'subcommand', description: '开机启动', priority: 80, usage: 'systemctl enable nginx' },
        { text: 'disable', type: 'subcommand', description: '禁用启动', priority: 80, usage: 'systemctl disable nginx' },
        { text: 'list-units', type: 'subcommand', description: '列出服务', priority: 75, usage: 'systemctl list-units --type=service' },
        { text: 'daemon-reload', type: 'subcommand', description: '重载 daemon', priority: 70, usage: 'systemctl daemon-reload' },
        { text: 'is-active', type: 'subcommand', description: '检查是否运行', priority: 65 },
        { text: 'is-enabled', type: 'subcommand', description: '检查是否启用', priority: 65 }
    ],
    subcommands: {
        'start': systemctlStart,
        'stop': systemctlStop,
        'restart': systemctlRestart,
        'reload': systemctlRestart,
        'status': systemctlStatus,
        'enable': systemctlEnable,
        'disable': systemctlDisable,
        'list-units': systemctlListUnits,
        'daemon-reload': systemctlDaemonReload,
        'is-active': systemctlStatus,
        'is-enabled': systemctlStatus
    }
};

export default systemctlCommand;

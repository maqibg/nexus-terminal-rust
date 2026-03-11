
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getDockerContainers, getDockerImages, getDockerNetworks, getDockerVolumes, getRemoteDirectories, getRemoteFiles } from '../providers/file-system';

/**
 * Docker 命令定义
 * 支持智能读取运行中的容器和镜像
 */

// docker ps 子命令
const dockerPs: CommandDefinition = {
    name: 'ps',
    description: '列出容器',
    options: [
        { text: '-a', type: 'option', description: '显示所有容器（包括已停止）', priority: 100, usage: 'docker ps -a' },
        { text: '-q', type: 'option', description: '只显示容器 ID', priority: 90 },
        { text: '--filter', type: 'option', description: '过滤条件', priority: 80, usage: '--filter "status=running"' }
    ]
};

// docker run 子命令
const dockerRun: CommandDefinition = {
    name: 'run',
    description: '在新容器中运行命令',
    options: [
        { text: '-d', type: 'option', description: '后台运行', priority: 100 },
        { text: '-it', type: 'option', description: '交互式终端', priority: 95 },
        { text: '-p', type: 'option', description: '端口映射', priority: 90, usage: '-p 8080:80' },
        { text: '-v', type: 'option', description: '挂载卷', priority: 85, usage: '-v /host:/container' },
        { text: '--name', type: 'option', description: '容器名称', priority: 80, usage: '--name my-app' },
        { text: '--rm', type: 'option', description: '退出后删除', priority: 75 },
        { text: '-e', type: 'option', description: '环境变量', priority: 70, usage: '-e KEY=value' }
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        // 如果当前参数不是选项，尝试补全镜像名
        if (!ctx.currentArg.startsWith('-')) {
            const images = await getDockerImages(ctx.sessionId!, ctx.electronAPI);
            return images.filter(img =>
                !ctx.currentArg || img.text.includes(ctx.currentArg)
            );
        }
        return [];
    }
};

// docker exec 子命令
const dockerExec: CommandDefinition = {
    name: 'exec',
    description: '在运行的容器中执行命令',
    options: [
        { text: '-it', type: 'option', description: '交互式终端', priority: 100 },
        { text: '-d', type: 'option', description: '后台执行', priority: 80 },
        { text: '-u', type: 'option', description: '指定用户', priority: 70, usage: '-u root' },
        { text: '-w', type: 'option', description: '工作目录', priority: 65, usage: '-w /app' }
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        // 如果当前正在输入选项，返回空让静态选项匹配
        if (ctx.currentArg.startsWith('-')) {
            return [];
        }

        // ctx.args 包含整个命令: ['docker', 'exec', '-it', 'container', ...]
        // 我们需要找出 "exec" 之后的非选项参数
        const execIndex = ctx.args.indexOf('exec');
        if (execIndex === -1) return [];

        // 获取 exec 之后的参数（不包括当前正在输入的参数）
        const argsAfterExec = ctx.args.slice(execIndex + 1, -1); // 排除最后一个（当前输入）

        // 过滤出非选项参数（这些应该是容器名和命令）
        const nonOptionArgs = argsAfterExec.filter(arg => arg && !arg.startsWith('-'));

        if (nonOptionArgs.length === 0) {
            // 还没有容器名，补全容器
            const containers = await getDockerContainers(ctx.sessionId!, ctx.electronAPI, false);
            // 容器名后面加空格，方便继续输入命令
            return containers.map(c => ({
                ...c,
                text: c.text + ' ',  // 添加空格，继续补全命令
            }));
        } else if (nonOptionArgs.length === 1) {
            // 已有容器名，补全常用命令
            return [
                { text: '/bin/bash', type: 'hint' as const, description: 'Bash Shell', priority: 100, matchPart: '', restPart: '/bin/bash' },
                { text: '/bin/sh', type: 'hint' as const, description: 'Shell', priority: 95, matchPart: '', restPart: '/bin/sh' },
                { text: 'bash', type: 'hint' as const, description: 'Bash', priority: 90, matchPart: '', restPart: 'bash' },
                { text: 'sh', type: 'hint' as const, description: 'Shell', priority: 85, matchPart: '', restPart: 'sh' },
                { text: 'ls', type: 'hint' as const, description: '列出文件', priority: 80, matchPart: '', restPart: 'ls' },
                { text: 'cat', type: 'hint' as const, description: '查看文件', priority: 75, matchPart: '', restPart: 'cat' },
                { text: 'env', type: 'hint' as const, description: '环境变量', priority: 70, matchPart: '', restPart: 'env' }
            ];
        }
        // 已有容器名和命令，不再补全
        return [];
    }
};

// docker stop/start/restart 通用逻辑
const createContainerOperation = (name: string, desc: string, allContainers = false): CommandDefinition => ({
    name,
    description: desc,
    options: [],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const containers = await getDockerContainers(ctx.sessionId!, ctx.electronAPI, allContainers);
        return containers.filter(c =>
            !ctx.currentArg || c.text.includes(ctx.currentArg)
        );
    }
});

// docker logs 子命令
const dockerLogs: CommandDefinition = {
    name: 'logs',
    description: '获取容器日志',
    options: [
        { text: '-f', type: 'option', description: '实时跟踪日志', priority: 100 },
        { text: '--tail', type: 'option', description: '显示最后N行', priority: 90, usage: '--tail 100' },
        { text: '-t', type: 'option', description: '显示时间戳', priority: 80 }
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.currentArg.startsWith('-')) {
            return getDockerContainers(ctx.sessionId!, ctx.electronAPI, true);
        }
        return [];
    }
};

// docker build 子命令
const dockerBuild: CommandDefinition = {
    name: 'build',
    description: '构建镜像',
    options: [
        { text: '-t', type: 'option', description: '标记镜像', priority: 100, usage: '-t name:tag' },
        { text: '-f', type: 'option', description: 'Dockerfile 路径', priority: 90 },
        { text: '--no-cache', type: 'option', description: '不使用缓存', priority: 80 },
        { text: '.', type: 'path', description: '当前目录作为上下文', priority: 100 }
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        // 补全目录路径
        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteDirectories(ctx.sessionId!, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    }
};

// docker rm 子命令
const dockerRm: CommandDefinition = {
    name: 'rm',
    description: '删除容器',
    options: [
        { text: '-f', type: 'option', description: '强制删除', priority: 90 }
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        // 显示所有容器（包括已停止的）
        return getDockerContainers(ctx.sessionId!, ctx.electronAPI, true);
    }
};

// docker rmi 子命令
const dockerRmi: CommandDefinition = {
    name: 'rmi',
    description: '删除镜像',
    options: [
        { text: '-f', type: 'option', description: '强制删除', priority: 90 }
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        return getDockerImages(ctx.sessionId!, ctx.electronAPI);
    }
};

// docker pull 子命令
const dockerPull: CommandDefinition = {
    name: 'pull',
    description: '拉取镜像',
    options: [
        { text: 'nginx', type: 'hint', description: 'Web 服务器', priority: 90 },
        { text: 'mysql', type: 'hint', description: '数据库', priority: 85 },
        { text: 'redis', type: 'hint', description: '缓存', priority: 85 },
        { text: 'node', type: 'hint', description: 'Node.js', priority: 80 },
        { text: 'python', type: 'hint', description: 'Python', priority: 80 },
        { text: 'ubuntu', type: 'hint', description: 'Ubuntu 系统', priority: 75 }
    ]
};

// docker images 子命令
const dockerImages: CommandDefinition = {
    name: 'images',
    description: '列出镜像',
    options: [
        { text: '-a', type: 'option', description: '显示所有镜像（包括中间层）', priority: 95 },
        { text: '--digests', type: 'option', description: '显示摘要', priority: 90 },
        { text: '--no-trunc', type: 'option', description: '不截断输出', priority: 85 },
        { text: '-q', type: 'option', description: '只显示 ID', priority: 80 },
        { text: '--filter', type: 'option', description: '过滤条件', priority: 75, usage: '--filter dangling=true' },
        { text: '--format', type: 'option', description: '自定义输出格式', priority: 70, usage: '--format "{{.Repository}}:{{.Tag}}"' },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        if (!ctx.currentArg.startsWith('-')) {
            return getDockerImages(ctx.sessionId, ctx.electronAPI);
        }
        return [];
    }
};

// docker push 子命令
const dockerPush: CommandDefinition = {
    name: 'push',
    description: '推送镜像',
    options: [
        { text: '--all-tags', type: 'option', description: '推送所有 tag', priority: 90 },
        { text: '--disable-content-trust', type: 'option', description: '禁用内容信任', priority: 80 },
        { text: '--quiet', type: 'option', description: '静默输出', priority: 70 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        if (ctx.currentArg.startsWith('-')) return [];
        const images = await getDockerImages(ctx.sessionId, ctx.electronAPI);
        if (!ctx.currentArg) return images;
        return images.filter((img) => img.text.includes(ctx.currentArg));
    }
};

const dockerNetwork: CommandDefinition = {
    name: 'network',
    description: '管理网络',
    options: [
        { text: 'ls', type: 'subcommand', description: '列出网络', priority: 95 },
        { text: 'inspect', type: 'subcommand', description: '查看网络详情', priority: 90 },
        { text: 'create', type: 'subcommand', description: '创建网络', priority: 85 },
        { text: 'rm', type: 'subcommand', description: '删除网络', priority: 80 },
        { text: 'prune', type: 'subcommand', description: '清理未使用网络', priority: 75 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        if (ctx.currentArg.startsWith('-')) return [];

        const action = ctx.args[2] ?? '';
        if (action !== 'inspect' && action !== 'rm') {
            return [];
        }

        const networks = await getDockerNetworks(ctx.sessionId, ctx.electronAPI);
        if (!ctx.currentArg) return networks;
        return networks.filter((net) => net.text.includes(ctx.currentArg));
    },
};

const dockerVolume: CommandDefinition = {
    name: 'volume',
    description: '管理卷',
    options: [
        { text: 'ls', type: 'subcommand', description: '列出卷', priority: 95 },
        { text: 'inspect', type: 'subcommand', description: '查看卷详情', priority: 90 },
        { text: 'create', type: 'subcommand', description: '创建卷', priority: 85 },
        { text: 'rm', type: 'subcommand', description: '删除卷', priority: 80 },
        { text: 'prune', type: 'subcommand', description: '清理未使用卷', priority: 75 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        if (ctx.currentArg.startsWith('-')) return [];

        const action = ctx.args[2] ?? '';
        if (action !== 'inspect' && action !== 'rm') {
            return [];
        }

        const volumes = await getDockerVolumes(ctx.sessionId, ctx.electronAPI);
        if (!ctx.currentArg) return volumes;
        return volumes.filter((vol) => vol.text.includes(ctx.currentArg));
    },
};

// docker compose 子命令
const dockerCompose: CommandDefinition = {
    name: 'compose',
    description: 'Compose 命令',
    options: [
        { text: '-f', type: 'option', description: '指定 compose 文件', priority: 95, usage: '-f docker-compose.yml', repeatable: true },
        { text: '--file', type: 'option', description: '同 -f', priority: 95, repeatable: true },
        { text: '-p', type: 'option', description: '项目名', priority: 90, usage: '-p my-project' },
        { text: '--project-name', type: 'option', description: '项目名', priority: 90 },
        { text: '--profile', type: 'option', description: '指定 profile', priority: 85, usage: '--profile dev' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },

        { text: 'up', type: 'subcommand', description: '创建并启动', priority: 100, usage: 'docker compose up -d' },
        { text: 'down', type: 'subcommand', description: '停止并移除', priority: 95 },
        { text: 'ps', type: 'subcommand', description: '查看服务状态', priority: 92 },
        { text: 'logs', type: 'subcommand', description: '查看日志', priority: 90 },
        { text: 'exec', type: 'subcommand', description: '进入服务执行命令', priority: 88 },
        { text: 'build', type: 'subcommand', description: '构建镜像', priority: 86 },
        { text: 'pull', type: 'subcommand', description: '拉取镜像', priority: 84 },
        { text: 'push', type: 'subcommand', description: '推送镜像', priority: 82 },
        { text: 'restart', type: 'subcommand', description: '重启服务', priority: 80 },
        { text: 'stop', type: 'subcommand', description: '停止服务', priority: 78 },
        { text: 'start', type: 'subcommand', description: '启动服务', priority: 76 },
        { text: 'rm', type: 'subcommand', description: '移除已停止容器', priority: 74 },
        { text: 'run', type: 'subcommand', description: '运行一次性命令', priority: 72 },
        { text: 'config', type: 'subcommand', description: '输出配置', priority: 70 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        const prev = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (!ctx.sessionId || !ctx.electronAPI) return [];
        if (prev === '-f' || prev === '--file') {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg || './', ctx.electronAPI);
        }
        return [];
    },
};

// 主 docker 命令
const dockerCommand: CommandDefinition = {
    name: 'docker',
    description: '应用容器引擎',
    options: [
        { text: 'ps', type: 'subcommand', description: '列出容器', priority: 100, usage: 'docker ps -a' },
        { text: 'run', type: 'subcommand', description: '运行新容器', priority: 95, usage: 'docker run -d nginx' },
        { text: 'exec', type: 'subcommand', description: '进入容器执行命令', priority: 95, usage: 'docker exec -it <容器> bash' },
        { text: 'logs', type: 'subcommand', description: '查看容器日志', priority: 90, usage: 'docker logs -f <容器>' },
        { text: 'stop', type: 'subcommand', description: '停止容器', priority: 90, usage: 'docker stop <容器>' },
        { text: 'start', type: 'subcommand', description: '启动容器', priority: 85, usage: 'docker start <容器>' },
        { text: 'restart', type: 'subcommand', description: '重启容器', priority: 85, usage: 'docker restart <容器>' },
        { text: 'rm', type: 'subcommand', description: '删除容器', priority: 80, usage: 'docker rm <容器>' },
        { text: 'rmi', type: 'subcommand', description: '删除镜像', priority: 80, usage: 'docker rmi <镜像>' },
        { text: 'images', type: 'subcommand', description: '列出镜像', priority: 80, usage: 'docker images' },
        { text: 'build', type: 'subcommand', description: '构建镜像', priority: 75, usage: 'docker build -t name .' },
        { text: 'pull', type: 'subcommand', description: '拉取镜像', priority: 75, usage: 'docker pull nginx' },
        { text: 'push', type: 'subcommand', description: '推送镜像', priority: 70, usage: 'docker push repo/image' },
        { text: 'network', type: 'subcommand', description: '管理网络', priority: 65 },
        { text: 'volume', type: 'subcommand', description: '管理卷', priority: 65 },
        { text: 'compose', type: 'subcommand', description: 'Compose 命令', priority: 60, usage: 'docker compose up -d' }
    ],
    subcommands: {
        'ps': dockerPs,
        'run': dockerRun,
        'exec': dockerExec,
        'stop': createContainerOperation('stop', '停止容器'),
        'start': createContainerOperation('start', '启动容器', true),
        'restart': createContainerOperation('restart', '重启容器'),
        'logs': dockerLogs,
        'build': dockerBuild,
        'rm': dockerRm,
        'rmi': dockerRmi,
        'pull': dockerPull,
        'images': dockerImages,
        'push': dockerPush,
        'network': dockerNetwork,
        'volume': dockerVolume,
        'compose': dockerCompose,
    }
};

export default dockerCommand;

import type { CommandDefinition } from '../types';

/**
 * tmux 命令定义（常用子命令）
 */

const tmuxNewSession: CommandDefinition = {
    name: 'new-session',
    description: '创建会话',
    options: [
        { text: '-s', type: 'option', description: '会话名', priority: 100, usage: 'tmux new -s name' },
        { text: '-d', type: 'option', description: '后台创建', priority: 95, usage: 'tmux new -d -s name' },
        { text: '-A', type: 'option', description: '存在则直接 attach', priority: 90, usage: 'tmux new -A -s name' },
        { text: '-c', type: 'option', description: '起始目录', priority: 85, usage: 'tmux new -c /path' },
        { text: '-n', type: 'option', description: '窗口名', priority: 80, usage: 'tmux new -n win' },
    ],
};

const tmuxAttachSession: CommandDefinition = {
    name: 'attach-session',
    description: '附加会话',
    options: [
        { text: '-t', type: 'option', description: '目标会话', priority: 100, usage: 'tmux attach -t name' },
    ],
};

const tmuxListSessions: CommandDefinition = {
    name: 'list-sessions',
    description: '列出会话',
    options: [],
};

const tmuxKillSession: CommandDefinition = {
    name: 'kill-session',
    description: '结束会话',
    options: [
        { text: '-t', type: 'option', description: '目标会话', priority: 100, usage: 'tmux kill-session -t name' },
    ],
};

const tmuxKillServer: CommandDefinition = {
    name: 'kill-server',
    description: '结束 tmux 服务',
    options: [],
};

const tmuxRenameSession: CommandDefinition = {
    name: 'rename-session',
    description: '重命名会话',
    options: [
        { text: '-t', type: 'option', description: '目标会话', priority: 95, usage: 'tmux rename-session -t old new' },
    ],
};

const tmuxCommand: CommandDefinition = {
    name: 'tmux',
    description: '终端复用器',
    options: [
        { text: 'new-session', type: 'subcommand', description: '创建会话', priority: 100, usage: 'tmux new-session -s name' },
        { text: 'new', type: 'subcommand', description: 'new-session (简写)', priority: 95, usage: 'tmux new -s name' },
        { text: 'attach-session', type: 'subcommand', description: '附加会话', priority: 90, usage: 'tmux attach-session -t name' },
        { text: 'attach', type: 'subcommand', description: 'attach-session (简写)', priority: 88, usage: 'tmux attach -t name' },
        { text: 'list-sessions', type: 'subcommand', description: '列出会话', priority: 85, usage: 'tmux list-sessions' },
        { text: 'ls', type: 'subcommand', description: 'list-sessions (简写)', priority: 83, usage: 'tmux ls' },
        { text: 'kill-session', type: 'subcommand', description: '结束会话', priority: 80, usage: 'tmux kill-session -t name' },
        { text: 'kill-server', type: 'subcommand', description: '结束服务', priority: 78, usage: 'tmux kill-server' },
        { text: 'rename-session', type: 'subcommand', description: '重命名会话', priority: 75, usage: 'tmux rename-session -t old new' },
        { text: '-V', type: 'option', description: '显示版本', priority: 55, usage: 'tmux -V' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
    subcommands: {
        'new-session': tmuxNewSession,
        'new': tmuxNewSession,
        'attach-session': tmuxAttachSession,
        'attach': tmuxAttachSession,
        'list-sessions': tmuxListSessions,
        'ls': tmuxListSessions,
        'kill-session': tmuxKillSession,
        'kill-server': tmuxKillServer,
        'rename-session': tmuxRenameSession,
    },
};

export default tmuxCommand;

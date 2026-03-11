import type { CommandDefinition } from '../types';

/**
 * ssh-keygen 命令定义（常用参数）
 */
const sshKeygenCommand: CommandDefinition = {
    name: 'ssh-keygen',
    description: 'SSH 密钥管理',
    options: [
        { text: '-t', type: 'option', description: '密钥类型', priority: 100, usage: 'ssh-keygen -t ed25519' },
        { text: '-b', type: 'option', description: '密钥位数', priority: 95, usage: 'ssh-keygen -b 4096' },
        { text: '-C', type: 'option', description: '备注(comment)', priority: 90, usage: 'ssh-keygen -C \"me@example.com\"' },
        { text: '-f', type: 'option', description: '输出文件', priority: 88, usage: 'ssh-keygen -f ~/.ssh/id_ed25519' },
        { text: '-N', type: 'option', description: '新口令', priority: 85, usage: 'ssh-keygen -N \"\"' },
        { text: '-p', type: 'option', description: '修改口令', priority: 80, usage: 'ssh-keygen -p -f ~/.ssh/id_ed25519' },
        { text: '-l', type: 'option', description: '显示指纹', priority: 75, usage: 'ssh-keygen -l -f ~/.ssh/id_ed25519.pub' },
        { text: '-R', type: 'option', description: '从 known_hosts 移除', priority: 70, usage: 'ssh-keygen -R host' },
        { text: '-F', type: 'option', description: '在 known_hosts 查找', priority: 65, usage: 'ssh-keygen -F host' },
        { text: '-A', type: 'option', description: '生成主机密钥', priority: 60, usage: 'ssh-keygen -A' },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default sshKeygenCommand;

import type { CommandDefinition } from '../types';

/**
 * ssh-copy-id 命令定义（常用参数）
 */
const sshCopyIdCommand: CommandDefinition = {
    name: 'ssh-copy-id',
    description: '复制公钥到远端 authorized_keys',
    options: [
        { text: '-i', type: 'option', description: '指定公钥文件', priority: 100, usage: 'ssh-copy-id -i ~/.ssh/id_ed25519.pub user@host' },
        { text: '-p', type: 'option', description: '端口', priority: 95, usage: 'ssh-copy-id -p 22 user@host' },
        { text: '-o', type: 'option', description: 'ssh 配置项', priority: 90 },
        { text: '-F', type: 'option', description: 'ssh config 文件', priority: 85 },
        { text: '-f', type: 'option', description: '强制追加', priority: 80 },
        { text: '-n', type: 'option', description: '仅显示，不执行', priority: 75 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default sshCopyIdCommand;

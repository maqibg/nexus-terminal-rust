import type { CommandDefinition } from '../types';

/**
 * sftp 命令定义（常用参数）
 */
const sftpCommand: CommandDefinition = {
    name: 'sftp',
    description: '交互式文件传输 (SFTP)',
    options: [
        { text: '-P', type: 'option', description: '端口', priority: 100, usage: 'sftp -P 22 user@host' },
        { text: '-i', type: 'option', description: '身份文件', priority: 95, usage: 'sftp -i ~/.ssh/id_rsa user@host' },
        { text: '-o', type: 'option', description: 'ssh 配置项', priority: 90, usage: 'sftp -o StrictHostKeyChecking=no user@host' },
        { text: '-F', type: 'option', description: 'ssh config 文件', priority: 85, usage: 'sftp -F ~/.ssh/config host' },
        { text: '-b', type: 'option', description: 'batch 文件', priority: 80, usage: 'sftp -b batch.txt user@host' },
        { text: '-q', type: 'option', description: '安静模式', priority: 75 },
        { text: '-C', type: 'option', description: '启用压缩', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default sftpCommand;

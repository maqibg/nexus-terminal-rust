
import type { CommandDefinition } from '../types';

/**
 * hostnamectl 命令定义
 */
const hostnamectlCommand: CommandDefinition = {
    name: 'hostnamectl',
    description: '主机名与系统信息管理',
    options: [
        { text: 'status', type: 'subcommand', description: '查看状态', priority: 100, usage: 'hostnamectl status' },
        { text: 'set-hostname', type: 'subcommand', description: '设置主机名', priority: 95, usage: 'hostnamectl set-hostname my-host' },
        { text: '--static', type: 'option', description: '静态主机名', priority: 80 },
        { text: '--transient', type: 'option', description: '临时主机名', priority: 75 },
        { text: '--pretty', type: 'option', description: '美化主机名', priority: 70 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 50 },
    ],
};

export default hostnamectlCommand;


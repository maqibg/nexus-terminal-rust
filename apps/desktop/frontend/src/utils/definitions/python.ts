
import type { CommandDefinition, CompletionContext, CompletionItem } from '../types';
import { getRemoteFiles } from '../providers/file-system';

/**
 * Python 命令定义
 */

const PYTHON_MODULE_HINTS: CompletionItem[] = [
    { text: 'pip', type: 'subcommand', description: '包管理器 (python -m pip ...)', priority: 95, matchPart: '', restPart: 'pip' },
    { text: 'venv', type: 'subcommand', description: '虚拟环境 (python -m venv .venv)', priority: 92, matchPart: '', restPart: 'venv' },
    { text: 'http.server', type: 'subcommand', description: '静态文件服务器', priority: 90, matchPart: '', restPart: 'http.server' },
    { text: 'json.tool', type: 'subcommand', description: 'JSON 格式化/校验', priority: 85, matchPart: '', restPart: 'json.tool' },
    { text: 'unittest', type: 'subcommand', description: '单元测试', priority: 80, matchPart: '', restPart: 'unittest' },
    { text: 'pdb', type: 'subcommand', description: '调试器', priority: 75, matchPart: '', restPart: 'pdb' },
    { text: 'timeit', type: 'subcommand', description: '简单性能测试', priority: 70, matchPart: '', restPart: 'timeit' },
];

function filterHints(items: CompletionItem[], prefix: string): CompletionItem[] {
    if (!prefix) return items;
    return items.filter((item) => item.text.startsWith(prefix));
}

const pythonCommand: CommandDefinition = {
    name: 'python',
    description: 'Python 解释器',
    options: [
        { text: '-m', type: 'option', description: '以模块方式运行', priority: 100, usage: 'python -m http.server' },
        { text: '-c', type: 'option', description: '执行一段代码', priority: 95, usage: 'python -c "print(123)"' },
        { text: '-i', type: 'option', description: '执行后进入交互模式', priority: 90 },
        { text: '-u', type: 'option', description: '无缓冲输出', priority: 85 },
        { text: '-O', type: 'option', description: '优化模式', priority: 80 },
        { text: '-B', type: 'option', description: '不生成 .pyc', priority: 75 },
        { text: '-S', type: 'option', description: '不导入 site', priority: 70 },
        { text: '-E', type: 'option', description: '忽略环境变量', priority: 65 },
        { text: '-V', type: 'option', description: '显示版本', priority: 60, usage: 'python -V' },
        { text: '--version', type: 'option', description: '显示版本', priority: 60 },
        { text: '-h', type: 'option', description: '显示帮助', priority: 55 },
        { text: '--help', type: 'option', description: '显示帮助', priority: 55 },
    ],
    generate: async (ctx: CompletionContext): Promise<CompletionItem[]> => {
        if (!ctx.sessionId || !ctx.electronAPI) return [];

        const prevArg = ctx.args[ctx.currentArgIndex - 1] ?? '';
        if (prevArg === '-m') {
            return filterHints(PYTHON_MODULE_HINTS, ctx.currentArg).map((item) => ({
                ...item,
                matchPart: ctx.currentArg,
                restPart: item.text.substring(ctx.currentArg.length),
            }));
        }

        if (!ctx.currentArg.startsWith('-')) {
            return getRemoteFiles(ctx.sessionId, ctx.currentArg, ctx.electronAPI);
        }
        return [];
    },
};

export default pythonCommand;


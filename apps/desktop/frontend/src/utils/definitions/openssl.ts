import type { CommandDefinition } from '../types';

/**
 * openssl 命令定义（常用子命令）
 */

const opensslEnc: CommandDefinition = {
    name: 'enc',
    description: '对称加密/解密',
    options: [
        { text: '-e', type: 'option', description: '加密', priority: 95 },
        { text: '-d', type: 'option', description: '解密', priority: 95 },
        { text: '-base64', type: 'option', description: 'Base64 编码', priority: 90 },
        { text: '-in', type: 'option', description: '输入文件', priority: 85 },
        { text: '-out', type: 'option', description: '输出文件', priority: 80 },
        { text: '-k', type: 'option', description: '口令', priority: 75 },
        { text: '-pass', type: 'option', description: '口令来源', priority: 70, usage: '-pass pass:xxx' },
    ],
};

const opensslX509: CommandDefinition = {
    name: 'x509',
    description: 'X.509 证书处理',
    options: [
        { text: '-in', type: 'option', description: '输入证书', priority: 95 },
        { text: '-out', type: 'option', description: '输出证书', priority: 90 },
        { text: '-noout', type: 'option', description: '不输出 PEM', priority: 85 },
        { text: '-text', type: 'option', description: '输出详情', priority: 80 },
        { text: '-fingerprint', type: 'option', description: '指纹', priority: 75 },
        { text: '-subject', type: 'option', description: 'Subject', priority: 70 },
        { text: '-issuer', type: 'option', description: 'Issuer', priority: 70 },
        { text: '-dates', type: 'option', description: '有效期', priority: 65 },
    ],
};

const opensslReq: CommandDefinition = {
    name: 'req',
    description: 'CSR/自签证书',
    options: [
        { text: '-new', type: 'option', description: '生成 CSR', priority: 95 },
        { text: '-x509', type: 'option', description: '生成自签证书', priority: 90 },
        { text: '-in', type: 'option', description: '输入', priority: 85 },
        { text: '-out', type: 'option', description: '输出', priority: 85 },
        { text: '-key', type: 'option', description: '私钥', priority: 80 },
        { text: '-keyout', type: 'option', description: '输出私钥', priority: 75 },
        { text: '-subj', type: 'option', description: 'Subject', priority: 70, usage: '-subj \"/CN=example.com\"' },
        { text: '-nodes', type: 'option', description: '私钥不加密', priority: 65 },
    ],
};

const opensslPkcs12: CommandDefinition = {
    name: 'pkcs12',
    description: 'PKCS#12 证书包',
    options: [
        { text: '-export', type: 'option', description: '导出 p12', priority: 95 },
        { text: '-in', type: 'option', description: '输入证书', priority: 90 },
        { text: '-inkey', type: 'option', description: '输入私钥', priority: 88 },
        { text: '-out', type: 'option', description: '输出文件', priority: 85 },
        { text: '-passout', type: 'option', description: '输出口令', priority: 80, usage: '-passout pass:xxx' },
    ],
};

const opensslSClient: CommandDefinition = {
    name: 's_client',
    description: 'TLS 客户端',
    options: [
        { text: '-connect', type: 'option', description: '目标 host:port', priority: 95, usage: '-connect example.com:443' },
        { text: '-servername', type: 'option', description: 'SNI', priority: 90, usage: '-servername example.com' },
        { text: '-showcerts', type: 'option', description: '显示证书链', priority: 85 },
    ],
};

const opensslCommand: CommandDefinition = {
    name: 'openssl',
    description: 'SSL/TLS 与证书工具集',
    options: [
        { text: 'enc', type: 'subcommand', description: '对称加密/解密', priority: 100 },
        { text: 'x509', type: 'subcommand', description: '证书处理', priority: 95 },
        { text: 'req', type: 'subcommand', description: 'CSR/证书请求', priority: 92 },
        { text: 'pkcs12', type: 'subcommand', description: 'PKCS#12', priority: 90 },
        { text: 's_client', type: 'subcommand', description: 'TLS 客户端', priority: 88 },
        { text: 'version', type: 'subcommand', description: '显示版本', priority: 70, usage: 'openssl version -a' },
        { text: 'help', type: 'subcommand', description: '显示帮助', priority: 60, usage: 'openssl help' },
    ],
    subcommands: {
        enc: opensslEnc,
        x509: opensslX509,
        req: opensslReq,
        pkcs12: opensslPkcs12,
        s_client: opensslSClient,
        version: { name: 'version', description: '显示版本', options: [] },
        help: { name: 'help', description: '显示帮助', options: [] },
    },
};

export default opensslCommand;

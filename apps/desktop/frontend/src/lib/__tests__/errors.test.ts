import { describe, expect, it } from 'vitest';
import { prettifySftpMessage, sanitizeMessage, toAppError } from '../errors';

describe('prettifySftpMessage', () => {
  it('turns a classified sftp failure into a readable line', () => {
    const raw = "permission-denied: readdir failed at '/var/log': Permission denied (os error 13)";

    expect(prettifySftpMessage(raw)).toBe(
      '权限不足（/var/log）：readdir — Permission denied (os error 13)',
    );
  });

  it('keeps unknown tokens readable instead of dropping them', () => {
    const raw = "too-large: read failed at '/tmp/big.log': file is 90000000 bytes";
    expect(prettifySftpMessage(raw)).toContain('文件过大');
    expect(prettifySftpMessage(raw)).toContain('/tmp/big.log');
  });

  it('leaves messages without a category untouched', () => {
    expect(prettifySftpMessage('session not found')).toBe('session not found');
  });
});

describe('toAppError', () => {
  it('prettifies sftp errors but not other kinds', () => {
    const sftp = toAppError({
      kind: 'Sftp',
      message: "not-found: stat failed at '/etc/missing': No such file",
    });
    expect(sftp.kind).toBe('Sftp');
    expect(sftp.message).toBe('不存在（/etc/missing）：stat — No such file');

    const ssh = toAppError({
      kind: 'Ssh',
      message: "not-found: stat failed at '/etc/missing': No such file",
    });
    expect(ssh.message).toBe("not-found: stat failed at '/etc/missing': No such file");
  });

  it('redacts secrets before display', () => {
    expect(sanitizeMessage('password= hunter2')).toContain('[REDACTED]');
  });
});

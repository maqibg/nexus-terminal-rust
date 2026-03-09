import { describe, expect, it } from 'vitest';
import { buildRemoteUploadPath } from '../local-upload';
import type { LocalUploadEntry } from '../api-sftp';

function joinRemotePath(basePath: string, childName: string): string {
  const safeChild = childName.trim().replace(/^\/+/, '');
  const normalizedBase = basePath === '/' ? '/' : basePath.replace(/\/+$/, '');
  return normalizedBase === '/' ? `/${safeChild}` : `${normalizedBase}/${safeChild}`;
}

describe('buildRemoteUploadPath', () => {
  it('keeps top-level folder segments from dropped directories', () => {
    const entry: LocalUploadEntry = {
      local_path: 'C:\\demo\\logs\\nginx\\access.log',
      relative_path: 'logs/nginx/',
      display_path: 'logs/nginx/access.log',
    };

    expect(buildRemoteUploadPath('/var/www', entry, joinRemotePath))
      .toBe('/var/www/logs/nginx/access.log');
  });

  it('uploads dropped files directly into the current directory', () => {
    const entry: LocalUploadEntry = {
      local_path: 'C:\\demo\\README.md',
      relative_path: '',
      display_path: 'README.md',
    };

    expect(buildRemoteUploadPath('/', entry, joinRemotePath)).toBe('/README.md');
  });
});

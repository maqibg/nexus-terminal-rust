import { useFileEditorStore, type FileTab } from '@/stores/fileEditor';
import { sftpApi } from '@/lib/api';
import { useAlertDialog } from './useAlertDialog';
import { toAppError } from '@/lib/errors';

/**
 * Remote file editor composable — open/save/close via SFTP.
 */
export function useFileEditor() {
  const store = useFileEditorStore();
  const { alert } = useAlertDialog();

  function decodeUtf8Base64(base64: string): string {
    const bytes = Uint8Array.from(atob(base64), (char) => char.charCodeAt(0));
    return new TextDecoder('utf-8').decode(bytes);
  }

  function guessLanguage(path: string): string {
    const ext = path.split('.').pop()?.toLowerCase() ?? '';
    const map: Record<string, string> = {
      js: 'javascript', ts: 'typescript', json: 'json', py: 'python',
      sh: 'shell', bash: 'shell', yml: 'yaml', yaml: 'yaml',
      md: 'markdown', html: 'html', css: 'css', xml: 'xml',
      rs: 'rust', toml: 'toml', sql: 'sql', conf: 'ini', ini: 'ini',
    };
    return map[ext] ?? 'plaintext';
  }

  async function openFile(sessionId: string, path: string) {
    // Reuse existing tab
    const tabId = `${sessionId}:${path}`;
    const existing = store.openFiles.get(tabId);
    if (existing) {
      store.setActive(tabId);
      return;
    }

    try {
      const base64 = await sftpApi.readFile(sessionId, path);
      const content = decodeUtf8Base64(base64);
      const tab: FileTab = {
        id: tabId,
        sessionId,
        path,
        content,
        originalContent: content,
        isDirty: false,
        language: guessLanguage(path),
        rawContentBase64: base64,
        selectedEncoding: 'utf-8',
      };
      store.openFile(tab);
    } catch (e: unknown) {
      await alert('Open File Error', toAppError(e).message);
    }
  }

  async function saveFile(tabId: string) {
    const tab = store.openFiles.get(tabId);
    if (!tab) return;
    try {
      const base64 = btoa(tab.content);
      await sftpApi.writeFile(tab.sessionId, tab.path, base64);
      store.markSaved(tabId);
    } catch (e: unknown) {
      await alert('Save Error', toAppError(e).message);
    }
  }

  function closeTab(tabId: string) {
    store.closeFile(tabId);
  }

  return { openFile, saveFile, closeTab };
}

import { useFileEditorStore, type FileTab } from '@/stores/fileEditor';
import { sftpApi } from '@/lib/api';
import { useAlertDialog } from './useAlertDialog';

/**
 * Remote file editor composable — open/save/close via SFTP.
 */
export function useFileEditor() {
  const store = useFileEditorStore();
  const { alert } = useAlertDialog();

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
      const content = atob(base64);
      const tab: FileTab = {
        id: tabId,
        sessionId,
        path,
        content,
        originalContent: content,
        isDirty: false,
        language: guessLanguage(path),
      };
      store.openFile(tab);
    } catch (e: any) {
      await alert('Open File Error', e.message ?? String(e));
    }
  }

  async function saveFile(tabId: string) {
    const tab = store.openFiles.get(tabId);
    if (!tab) return;
    try {
      const base64 = btoa(tab.content);
      await sftpApi.writeFile(tab.sessionId, tab.path, base64);
      store.markSaved(tabId);
    } catch (e: any) {
      await alert('Save Error', e.message ?? String(e));
    }
  }

  function closeTab(tabId: string) {
    store.closeFile(tabId);
  }

  return { openFile, saveFile, closeTab };
}

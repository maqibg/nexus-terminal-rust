/**
 * SSH Suspend API — matches backend Tauri command signatures.
 */
import { tauriInvoke } from './invoke';

export interface SuspendedSession {
  id: string;
  connection_id: number;
  connection_name: string;
  suspended_at: string;
}

export const sshSuspendApi = {
  list: () => tauriInvoke<SuspendedSession[]>('ssh_suspend_list'),
  suspend: (sessionId: string) => tauriInvoke<void>('ssh_suspend', { sessionId }),
  resume: (id: string) => tauriInvoke<string>('ssh_resume', { id }),
  terminate: (id: string) => tauriInvoke<void>('ssh_suspend_terminate', { id }),
};

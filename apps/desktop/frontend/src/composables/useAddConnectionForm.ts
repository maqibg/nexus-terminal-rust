import { ref, reactive, watch } from 'vue';
import { connectionsApi } from '@/lib/api';
import { useAlertDialog } from './useAlertDialog';

export interface ConnectionFormData {
  name: string;
  type: 'SSH' | 'RDP';
  host: string;
  port: number;
  username: string;
  auth_method: string;
  password?: string;
  ssh_key_id?: number;
  proxy_id?: number;
  tags: string[];
  jump_chain?: string;
  encoding?: string;
  notes?: string;
}

function defaultForm(): ConnectionFormData {
  return {
    name: '',
    type: 'SSH',
    host: '',
    port: 22,
    username: 'root',
    auth_method: 'password',
    password: '',
    tags: [],
  };
}

function normalizePayload(formData: ConnectionFormData): Record<string, unknown> {
  const payload: Record<string, unknown> = {
    ...formData,
    type: formData.type,
  };

  if (formData.type === 'RDP') {
    payload.auth_method = 'none';
    delete payload.password;
    delete payload.ssh_key_id;
    return payload;
  }

  if (formData.auth_method !== 'password') {
    delete payload.password;
  }
  if (formData.auth_method !== 'key') {
    delete payload.ssh_key_id;
  }

  return payload;
}

/**
 * Connection form composable — create/edit/test.
 */
export function useAddConnectionForm(mode: 'create' | 'edit', connectionId?: number) {
  const formData = reactive<ConnectionFormData>(defaultForm());
  const loading = ref(false);
  const saving = ref(false);
  const { alert } = useAlertDialog();

  watch(
    () => formData.type,
    (value) => {
      if (value === 'RDP') {
        if (formData.port === 22) formData.port = 3389;
        if (!formData.username.trim()) formData.username = 'Administrator';
        formData.auth_method = 'none';
      } else if (value === 'SSH') {
        if (formData.port === 3389) formData.port = 22;
        if (formData.auth_method === 'none') formData.auth_method = 'password';
      }
    }
  );

  async function loadConnection() {
    if (mode !== 'edit' || !connectionId) return;
    loading.value = true;
    try {
      const conn = await connectionsApi.get(connectionId);
      Object.assign(formData, {
        name: conn.name,
        type: String(conn.type ?? 'SSH').toUpperCase() === 'RDP' ? 'RDP' : 'SSH',
        host: conn.host,
        port: conn.port,
        username: conn.username,
        auth_method: conn.auth_method,
        ssh_key_id: conn.ssh_key_id,
        proxy_id: conn.proxy_id,
        tags: conn.tags ?? [],
      });
    } catch (e: any) {
      await alert('Load Error', e.message ?? String(e));
    } finally {
      loading.value = false;
    }
  }

  function validate(): boolean {
    if (!formData.name.trim() || !formData.host.trim() || formData.port <= 0) {
      return false;
    }

    if (formData.type === 'SSH') {
      if (!formData.username.trim()) return false;
      if (formData.auth_method === 'password' && mode !== 'edit' && !formData.password?.trim()) {
        return false;
      }
      if (formData.auth_method === 'key' && !formData.ssh_key_id) {
        return false;
      }
    }

    return true;
  }

  async function save(): Promise<boolean> {
    if (!validate()) {
      await alert('Validation', '请完整填写必填项');
      return false;
    }

    saving.value = true;
    try {
      const payload = normalizePayload(formData);
      if (mode === 'edit' && connectionId) {
        await connectionsApi.update(connectionId, payload);
      } else {
        await connectionsApi.create(payload);
      }
      return true;
    } catch (e: any) {
      await alert('Save Error', e.message ?? String(e));
      return false;
    } finally {
      saving.value = false;
    }
  }

  async function testConnection(): Promise<boolean> {
    if (!validate()) {
      await alert('Validation', '请先完善连接配置');
      return false;
    }

    try {
      const payload = normalizePayload(formData);
      if (
        mode === 'edit' &&
        connectionId &&
        formData.type === 'SSH' &&
        formData.auth_method === 'password' &&
        !formData.password
      ) {
        return await connectionsApi.test(connectionId);
      }
      return await connectionsApi.testUnsaved(payload);
    } catch (e: any) {
      await alert('Test Failed', e.message ?? String(e));
      return false;
    }
  }

  return { formData, loading, saving, loadConnection, validate, save, testConnection };
}

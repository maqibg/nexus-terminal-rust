import { defineStore } from 'pinia';
import { ref } from 'vue';
import { connectionsApi, type SshKey } from '@/lib/api';

export const useSshKeysStore = defineStore('sshKeys', () => {
  const items = ref<SshKey[]>([]);
  const loading = ref(false);

  async function fetchAll() {
    loading.value = true;
    try { items.value = await connectionsApi.sshKeyList(); }
    finally { loading.value = false; }
  }

  async function create(name: string, privateKeyPem: string, passphrase?: string) {
    await connectionsApi.sshKeyCreate(name, privateKeyPem, passphrase);
    await fetchAll();
  }

  async function update(id: number, name: string, privateKeyPem?: string, passphrase?: string) {
    await connectionsApi.sshKeyUpdate(id, name, privateKeyPem, passphrase);
    await fetchAll();
  }

  async function remove(id: number) {
    await connectionsApi.sshKeyDelete(id);
    items.value = items.value.filter(k => k.id !== id);
  }

  return { items, loading, fetchAll, create, update, remove };
});

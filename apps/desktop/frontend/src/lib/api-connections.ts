/**
 * Connections API — matches backend Tauri command signatures.
 *
 * Backend signatures:
 *   connection_list()
 *   connection_get(id: i64)
 *   connection_create(input: ConnectionInput)
 *   connection_update(id: i64, input: ConnectionInput)
 *   connection_delete(id: i64)
 *   connection_reorder(req: ReorderRequest { ids })
 *   tag_list() / tag_create(req: TagCreateRequest { name }) / tag_delete(id: i64)
 *   ssh_key_list() / ssh_key_delete(id: i64)
 *   proxy_list() / proxy_delete(id: i64)
 */
import { tauriInvoke } from './invoke';

export interface Connection {
  id: number;
  name: string;
  type: string;
  host: string;
  port: number;
  username: string;
  auth_method: string;
  ssh_key_id?: number;
  proxy_id?: number;
  rdp_options?: string | null;
  vnc_options?: string | null;
  sort_order: number;
  tags: string[];
  [key: string]: unknown;
}

export interface Tag {
  id: number;
  name: string;
}

export interface SshKey {
  id: number;
  name: string;
}

export interface Proxy {
  id: number;
  name: string;
  proxy_type: string;
  host: string;
  port: number;
}

export const connectionsApi = {
  list: () => tauriInvoke<Connection[]>('connection_list'),
  get: (id: number) => tauriInvoke<Connection>('connection_get', { id }),
  create: (data: Record<string, unknown>) =>
    tauriInvoke<number>('connection_create', { input: data }),
  update: (id: number, data: Record<string, unknown>) =>
    tauriInvoke<boolean>('connection_update', { id, input: data }),
  delete: (id: number) => tauriInvoke<boolean>('connection_delete', { id }),
  reorder: (ids: number[]) =>
    tauriInvoke<void>('connection_reorder', { req: { ids } }),
  test: (id: number) => tauriInvoke<boolean>('connection_test', { id }),
  testUnsaved: (data: Record<string, unknown>) =>
    tauriInvoke<boolean>('connection_test_unsaved', { input: data }),
  clone: (id: number) => tauriInvoke<number>('connection_clone', { id }),
  export: (ids?: number[]) => tauriInvoke<string>('connection_export', { ids }),
  import: (json: string) => tauriInvoke<number[]>('connection_import', { json }),

  // Tags
  tagList: () => tauriInvoke<Tag[]>('tag_list'),
  tagCreate: (name: string) =>
    tauriInvoke<number>('tag_create', { req: { name } }),
  tagDelete: (id: number) => tauriInvoke<boolean>('tag_delete', { id }),

  // SSH Keys
  sshKeyList: () => tauriInvoke<SshKey[]>('ssh_key_list'),
  sshKeyCreate: (name: string, privateKeyPem: string, passphrase?: string) =>
    tauriInvoke<number>('ssh_key_create', { req: { name, private_key_pem: privateKeyPem, passphrase } }),
  sshKeyUpdate: (id: number, name: string, privateKeyPem?: string, passphrase?: string) =>
    tauriInvoke<boolean>('ssh_key_update', { req: { id, name, private_key_pem: privateKeyPem, passphrase } }),
  sshKeyDelete: (id: number) => tauriInvoke<boolean>('ssh_key_delete', { id }),

  // Proxies
  proxyList: () => tauriInvoke<Proxy[]>('proxy_list'),
  proxyCreate: (data: Record<string, unknown>) =>
    tauriInvoke<number>('proxy_create', { input: data }),
  proxyUpdate: (data: Record<string, unknown>) =>
    tauriInvoke<boolean>('proxy_update', { input: data }),
  proxyDelete: (id: number) => tauriInvoke<boolean>('proxy_delete', { id }),
};


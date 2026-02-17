import { computed, reactive, ref, watch, type Ref } from 'vue';
import { storeToRefs } from 'pinia';
import { connectionsApi, type Connection, type Proxy, type Tag } from '@/lib/api';
import { useConnectionsStore } from '@/stores/connections';
import { useProxiesStore } from '@/stores/proxies';
import { useTagsStore } from '@/stores/tags';
import { useSshKeysStore } from '@/stores/sshKeys';
import { useUiNotificationsStore } from '@/stores/uiNotifications';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { useAlertDialog } from '@/composables/useAlertDialog';

type ConnectionType = 'SSH' | 'RDP' | 'VNC';
type AuthMethod = 'password' | 'key';

interface ExtendedConnection extends Connection {
  jump_chain?: unknown;
  notes?: string | null;
  proxy_type?: 'proxy' | 'jump' | null;
}

interface ScriptLineResult {
  type: ConnectionType;
  username: string;
  host: string;
  port: number;
  name: string;
  password: string | null;
  keyName: string | null;
  proxyName: string | null;
  tagNames: string[];
  note: string;
}

interface BuildPayloadOptions {
  type: ConnectionType;
  name: string;
  host: string;
  port: number;
  username: string;
  authMethod: AuthMethod;
  password?: string;
  sshKeyId?: number | null;
  proxyId?: number | null;
  jumpChain?: Array<number | null> | null;
  tagNames: string[];
  notes: string;
}

export interface ConnectionFormData {
  id?: number;
  type: ConnectionType;
  name: string;
  host: string;
  port: number;
  username: string;
  auth_method: AuthMethod;
  password: string;
  selected_ssh_key_id: number | null;
  proxy_id: number | null;
  jump_chain: Array<number | null> | null;
  proxy_type: 'proxy' | 'jump' | null;
  tag_ids: number[];
  notes: string;
  vncPassword: string;
}

const scriptModeFormatInfo = `格式: user@host:port [-type TYPE] [-name NAME] [-p PASSWORD] [-k KEY_NAME] [-proxy PROXY_NAME] [-tags TAG1,TAG2] [-note NOTE]\n参数说明:\n  user@host:port  用户名@主机/IP:端口 (必填)\n  -type TYPE      连接类型，支持 SSH/RDP/VNC\n  -name NAME      显示名称\n  -p PASSWORD     密码\n  -k KEY_NAME     SSH 密钥名称\n  -proxy NAME     代理名称\n  -tags TAGS      标签（逗号分隔）\n  -note TEXT      备注`;

const createDefaultFormData = (): ConnectionFormData => ({
  type: 'SSH',
  name: '',
  host: '',
  port: 22,
  username: '',
  auth_method: 'password',
  password: '',
  selected_ssh_key_id: null,
  proxy_id: null,
  jump_chain: null,
  proxy_type: null,
  tag_ids: [],
  notes: '',
  vncPassword: '',
});

export function useAddConnectionForm(
  mode: Ref<'create' | 'edit'>,
  connectionId: Ref<number | undefined>,
  visible: Ref<boolean>,
  onSaved: () => void,
  onClose: () => void,
) {
  const formData = reactive<ConnectionFormData>(createDefaultFormData());
  const isSubmitting = ref(false);
  const isInitializing = ref(false);
  const testStatus = ref<'idle' | 'testing' | 'success' | 'error'>('idle');
  const testResult = ref<string | null>(null);
  const testLatency = ref<number | null>(null);
  const isScriptModeActive = ref(false);
  const scriptInputText = ref('');
  const advancedConnectionMode = ref<'proxy' | 'jump'>('proxy');
  const proxyStoreError = ref<string | null>(null);
  const tagStoreError = ref<string | null>(null);

  const notify = useUiNotificationsStore();
  const { confirm } = useConfirmDialog();
  const { alert } = useAlertDialog();

  const connectionsStore = useConnectionsStore();
  const proxiesStore = useProxiesStore();
  const tagsStore = useTagsStore();
  const sshKeysStore = useSshKeysStore();

  const { list: connectionsList, loading: isConnectionsLoading } = storeToRefs(connectionsStore);
  const { items: proxyItems, loading: isProxyLoading } = storeToRefs(proxiesStore);
  const { items: tagItems, loading: isTagLoading } = storeToRefs(tagsStore);
  const { items: sshKeyItems, loading: isSshKeyLoading } = storeToRefs(sshKeysStore);

  const isEditMode = computed(() => mode.value === 'edit' && Boolean(connectionId.value));
  const formTitle = computed(() => (isEditMode.value ? '编辑连接' : '添加新连接'));
  const submitButtonText = computed(() => {
    if (isSubmitting.value) {
      return isEditMode.value ? '保存中...' : '创建中...';
    }
    return isEditMode.value ? '确认保存' : '确认添加';
  });
  const testButtonText = computed(() => (testStatus.value === 'testing' ? '测试中...' : '测试连接'));
  const latencyColor = computed(() => {
    if (testLatency.value === null) {
      return 'inherit';
    }
    if (testLatency.value < 100) {
      return 'var(--green)';
    }
    if (testLatency.value < 500) {
      return 'var(--yellow)';
    }
    return 'var(--red)';
  });
  const isLoading = computed(() => (
    isInitializing.value
    || isSubmitting.value
    || isConnectionsLoading.value
    || isProxyLoading.value
    || isTagLoading.value
    || isSshKeyLoading.value
  ));

  const proxies = computed<Proxy[]>(() => proxyItems.value as Proxy[]);
  const tags = computed<Tag[]>(() => tagItems.value as Tag[]);
  const connections = computed<ExtendedConnection[]>(() => connectionsList.value as ExtendedConnection[]);

  const getErrorMessage = (error: unknown): string => {
    if (error instanceof Error && error.message) {
      return error.message;
    }
    if (typeof error === 'string' && error.trim().length > 0) {
      return error;
    }
    return '未知错误';
  };

  const mapTagNamesToIds = (names: string[]): number[] => {
    return names
      .map(name => tags.value.find(tag => tag.name === name)?.id)
      .filter((id): id is number => typeof id === 'number');
  };

  const mapTagIdsToNames = (ids: number[]): string[] => {
    return ids
      .map(id => tags.value.find(tag => tag.id === id)?.name)
      .filter((name): name is string => Boolean(name));
  };

  const parseJumpChain = (value: unknown): Array<number | null> | null => {
    if (!value) {
      return null;
    }

    if (Array.isArray(value)) {
      return value.map((item) => {
        const parsed = Number(item);
        return Number.isNaN(parsed) ? null : parsed;
      });
    }

    if (typeof value === 'string') {
      try {
        const parsed = JSON.parse(value);
        if (!Array.isArray(parsed)) {
          return null;
        }
        return parsed.map((item) => {
          const numeric = Number(item);
          return Number.isNaN(numeric) ? null : numeric;
        });
      } catch {
        return null;
      }
    }

    return null;
  };

  const resetForm = () => {
    Object.assign(formData, createDefaultFormData());
    testStatus.value = 'idle';
    testResult.value = null;
    testLatency.value = null;
    isScriptModeActive.value = false;
    scriptInputText.value = '';
    advancedConnectionMode.value = 'proxy';
  };

  const normalizeType = (value: unknown): ConnectionType => {
    const upper = String(value ?? 'SSH').toUpperCase();
    if (upper === 'RDP') return 'RDP';
    if (upper === 'VNC') return 'VNC';
    return 'SSH';
  };

  const applyConnectionToForm = (connection: ExtendedConnection) => {
    const connType = normalizeType(connection.type);
    const jumpChain = parseJumpChain(connection.jump_chain);

    formData.id = connection.id;
    formData.type = connType;
    formData.name = String(connection.name ?? '');
    formData.host = String(connection.host ?? '');
    formData.port = Number(connection.port ?? (connType === 'RDP' ? 3389 : connType === 'VNC' ? 5900 : 22));
    formData.username = String(connection.username ?? (connType === 'RDP' ? 'Administrator' : ''));
    formData.auth_method = connection.auth_method === 'key' ? 'key' : 'password';
    formData.password = '';
    formData.vncPassword = '';
    formData.selected_ssh_key_id = typeof connection.ssh_key_id === 'number' ? connection.ssh_key_id : null;
    formData.proxy_id = typeof connection.proxy_id === 'number' ? connection.proxy_id : null;
    formData.jump_chain = jumpChain;
    formData.proxy_type = jumpChain && jumpChain.length > 0 ? 'jump' : 'proxy';
    formData.notes = String(connection.notes ?? '');
    formData.tag_ids = mapTagNamesToIds(Array.isArray(connection.tags) ? connection.tags : []);

    advancedConnectionMode.value = jumpChain && jumpChain.length > 0 ? 'jump' : 'proxy';
  };

  const parseIpRange = (host: string): { ips: string[] | null; error?: string } => {
    if (!host.includes('~')) {
      return { ips: null };
    }

    const [startIpRaw, endIpRaw] = host.split('~').map(item => item.trim());
    const startIp = startIpRaw.split('.');
    const endIp = endIpRaw.split('.');
    if (startIp.length !== 4 || endIp.length !== 4) {
      return { ips: null, error: 'IP 范围格式无效' };
    }

    if (startIp[0] !== endIp[0] || startIp[1] !== endIp[1] || startIp[2] !== endIp[2]) {
      return { ips: null, error: 'IP 范围仅支持最后一段变化' };
    }

    const start = Number.parseInt(startIp[3], 10);
    const end = Number.parseInt(endIp[3], 10);
    if (Number.isNaN(start) || Number.isNaN(end) || start > end || start < 0 || end > 255) {
      return { ips: null, error: 'IP 范围无效' };
    }

    const prefix = `${startIp[0]}.${startIp[1]}.${startIp[2]}`;
    const ips: string[] = [];
    for (let index = start; index <= end; index += 1) {
      ips.push(`${prefix}.${index}`);
    }

    return { ips };
  };

  const buildPayload = (options: BuildPayloadOptions): Record<string, unknown> => {
    const payload: Record<string, unknown> = {
      type: options.type,
      name: options.name.trim() || `${options.username}@${options.host}`,
      host: options.host,
      port: options.port,
      username: options.username,
      notes: options.notes,
      tags: options.tagNames,
      proxy_id: options.proxyId ?? null,
      jump_chain: options.jumpChain && options.jumpChain.length > 0
        ? JSON.stringify(options.jumpChain.filter((item): item is number => typeof item === 'number'))
        : null,
    };

    if (options.type === 'SSH') {
      payload.auth_method = options.authMethod;
      if (options.authMethod === 'password') {
        if (options.password && options.password.trim().length > 0) {
          payload.password = options.password;
        }
      } else if (typeof options.sshKeyId === 'number') {
        payload.ssh_key_id = options.sshKeyId;
      }
      return payload;
    }

    payload.auth_method = 'password';
    if (options.password && options.password.trim().length > 0) {
      payload.password = options.password;
    }
    payload.ssh_key_id = null;
    payload.proxy_id = null;
    payload.jump_chain = null;
    return payload;
  };

  const ensureDependencies = async () => {
    proxyStoreError.value = null;
    tagStoreError.value = null;
    const results = await Promise.allSettled([
      connectionsStore.fetch(),
      proxiesStore.fetchAll(),
      tagsStore.fetchAll(),
      sshKeysStore.fetchAll(),
    ]);

    const proxyResult = results[1];
    if (proxyResult.status === 'rejected') {
      proxyStoreError.value = getErrorMessage(proxyResult.reason);
    }
    const tagResult = results[2];
    if (tagResult.status === 'rejected') {
      tagStoreError.value = getErrorMessage(tagResult.reason);
    }
  };

  const initializeForm = async () => {
    isInitializing.value = true;
    resetForm();
    try {
      await ensureDependencies();
      if (isEditMode.value && connectionId.value) {
        const conn = await connectionsApi.get(connectionId.value) as ExtendedConnection;
        applyConnectionToForm(conn);
      }
    } catch (error) {
      notify.addNotification('error', `加载连接配置失败: ${getErrorMessage(error)}`);
    } finally {
      isInitializing.value = false;
    }
  };

  watch(
    () => formData.type,
    (type, previousType) => {
      if (type === previousType) {
        return;
      }

      if (type === 'RDP' && (formData.port === 22 || formData.port === 5900)) {
        formData.port = 3389;
      }
      if (type === 'VNC' && (formData.port === 22 || formData.port === 3389)) {
        formData.port = 5900;
      }
      if (type === 'SSH' && (formData.port === 3389 || formData.port === 5900)) {
        formData.port = 22;
      }
      if (type !== 'SSH') {
        advancedConnectionMode.value = 'proxy';
      }
    },
  );

  watch(visible, async (isVisible) => {
    if (isVisible) {
      await initializeForm();
    }
  }, { immediate: true });

  watch([mode, connectionId], async () => {
    if (visible.value) {
      await initializeForm();
    }
  });

  const validateNormalForm = async (): Promise<boolean> => {
    if (!formData.host.trim()) {
      await alert('提示', '请填写主机 / IP');
      return false;
    }
    if (!formData.port || formData.port < 1 || formData.port > 65535) {
      await alert('提示', '端口必须在 1~65535 之间');
      return false;
    }
    if (!formData.username.trim()) {
      await alert('提示', '请填写用户名');
      return false;
    }

    if (formData.type === 'SSH') {
      if (formData.auth_method === 'password' && !isEditMode.value && !formData.password.trim()) {
        await alert('提示', 'SSH 密码不能为空');
        return false;
      }
      if (formData.auth_method === 'key' && !formData.selected_ssh_key_id) {
        await alert('提示', '请选择 SSH 密钥');
        return false;
      }
    }

    if (formData.type === 'RDP' && !isEditMode.value && !formData.password.trim()) {
      await alert('提示', 'RDP 密码不能为空');
      return false;
    }

    if (formData.type === 'VNC' && !isEditMode.value && !formData.vncPassword.trim()) {
      await alert('提示', 'VNC 密码不能为空');
      return false;
    }

    return true;
  };

  const createFromPayload = async (basePayload: Record<string, unknown>, hosts: string[]) => {
    const createTasks = hosts.map(async (host, index) => {
      const payload: Record<string, unknown> = { ...basePayload, host };
      const currentName = payload.name;
      if (hosts.length > 1 && typeof currentName === 'string' && currentName.trim().length > 0) {
        payload.name = `${currentName}-${index + 1}`;
      }
      await connectionsApi.create(payload);
    });
    await Promise.all(createTasks);
  };

  const handleScriptSubmit = async () => {
    const lines = scriptInputText.value.split('\n').map(line => line.trim()).filter(Boolean);
    if (lines.length === 0) {
      notify.addNotification('error', '脚本输入不能为空');
      return;
    }

    const parseLine = (line: string): ScriptLineResult => {
      const tokens = line.match(/"[^"]*"|'[^']*'|\S+/g)?.map(token => token.replace(/^['"]|['"]$/g, '')) ?? [];
      if (tokens.length === 0) {
        throw new Error('脚本行为空');
      }

      const first = tokens[0];
      const atIndex = first.indexOf('@');
      if (atIndex <= 0) {
        throw new Error(`缺少 user@host:port：${line}`);
      }

      const username = first.slice(0, atIndex);
      const hostPort = first.slice(atIndex + 1);
      const [hostPart, portText] = hostPort.split(':');
      if (!hostPart) {
        throw new Error(`主机格式无效：${line}`);
      }

      let type: ConnectionType = 'SSH';
      let name = '';
      let password: string | null = null;
      let keyName: string | null = null;
      let proxyName: string | null = null;
      let tagNames: string[] = [];
      let note = '';

      let cursor = 1;
      while (cursor < tokens.length) {
        const token = tokens[cursor];
        const key = token.toLowerCase();

        const readValue = () => {
          const value = tokens[cursor + 1];
          if (!value) {
            throw new Error(`参数 ${token} 缺少值`);
          }
          cursor += 2;
          return value;
        };

        if (key === '-type' || key === '--type' || key === '-t') {
          const value = readValue().toUpperCase();
          if (value === 'SSH' || value === 'RDP' || value === 'VNC') {
            type = value;
          } else {
            throw new Error(`连接类型无效: ${value}`);
          }
          continue;
        }
        if (key === '-name' || key === '--name' || key === '-n') {
          name = readValue();
          continue;
        }
        if (key === '-p' || key === '--password') {
          password = readValue();
          continue;
        }
        if (key === '-k' || key === '--key') {
          keyName = readValue();
          continue;
        }
        if (key === '-proxy' || key === '--proxy') {
          proxyName = readValue();
          continue;
        }
        if (key === '-tags' || key === '--tags') {
          tagNames = readValue().split(',').map(item => item.trim()).filter(Boolean);
          continue;
        }
        if (key === '-note' || key === '--note') {
          note = readValue();
          continue;
        }

        throw new Error(`未知参数: ${token}`);
      }

      const port = portText ? Number.parseInt(portText, 10) : (type === 'RDP' ? 3389 : type === 'VNC' ? 5900 : 22);
      if (Number.isNaN(port) || port <= 0 || port > 65535) {
        throw new Error(`端口无效: ${portText}`);
      }

      return {
        type,
        username,
        host: hostPart,
        port,
        name,
        password,
        keyName,
        proxyName,
        tagNames,
        note,
      };
    };

    isSubmitting.value = true;
    try {
      for (const line of lines) {
        const parsed = parseLine(line);

        if (parsed.type === 'SSH' && !parsed.password && !parsed.keyName) {
          throw new Error(`SSH 连接必须提供密码(-p)或密钥(-k): ${line}`);
        }
        if ((parsed.type === 'RDP' || parsed.type === 'VNC') && !parsed.password) {
          throw new Error(`${parsed.type} 连接必须提供密码(-p): ${line}`);
        }

        const keyId = parsed.keyName
          ? sshKeyItems.value.find(key => key.name === parsed.keyName)?.id
          : null;
        if (parsed.keyName && !keyId) {
          throw new Error(`未找到 SSH 密钥: ${parsed.keyName}`);
        }

        const proxyId = parsed.proxyName
          ? proxyItems.value.find(proxy => proxy.name === parsed.proxyName)?.id
          : null;
        if (parsed.proxyName && !proxyId) {
          throw new Error(`未找到代理: ${parsed.proxyName}`);
        }

        for (const tagName of parsed.tagNames) {
          const exists = tagItems.value.some(tag => tag.name === tagName);
          if (!exists) {
            await tagsStore.create(tagName);
          }
        }
        if (parsed.tagNames.length > 0) {
          await tagsStore.fetchAll();
        }

        const hostRangeResult = parseIpRange(parsed.host);
        if (hostRangeResult.error) {
          throw new Error(hostRangeResult.error);
        }
        const hosts = hostRangeResult.ips ?? [parsed.host];

        const payload = buildPayload({
          type: parsed.type,
          name: parsed.name || `${parsed.username}@${parsed.host}`,
          host: parsed.host,
          port: parsed.port,
          username: parsed.username,
          authMethod: parsed.keyName ? 'key' : 'password',
          password: parsed.type === 'VNC' ? parsed.password ?? '' : parsed.password ?? '',
          sshKeyId: keyId,
          proxyId,
          jumpChain: null,
          tagNames: parsed.tagNames,
          notes: parsed.note,
        });

        await createFromPayload(payload, hosts);
      }

      notify.addNotification('success', `脚本模式已添加 ${lines.length} 条连接配置`);
      emitSavedAndClose();
    } catch (error) {
      notify.addNotification('error', `脚本模式添加失败: ${getErrorMessage(error)}`);
    } finally {
      isSubmitting.value = false;
    }
  };

  const emitSavedAndClose = async () => {
    await connectionsStore.fetch();
    onSaved();
    onClose();
  };

  const handleSubmit = async () => {
    if (isSubmitting.value) {
      return;
    }

    if (isScriptModeActive.value && !isEditMode.value) {
      await handleScriptSubmit();
      return;
    }

    const valid = await validateNormalForm();
    if (!valid) {
      return;
    }

    isSubmitting.value = true;
    try {
      const hostResult = parseIpRange(formData.host.trim());
      if (hostResult.error) {
        notify.addNotification('error', hostResult.error);
        return;
      }

      const tagNames = mapTagIdsToNames(formData.tag_ids);
      const payload = buildPayload({
        type: formData.type,
        name: formData.name,
        host: formData.host.trim(),
        port: formData.port,
        username: formData.username.trim() || (formData.type === 'RDP' ? 'Administrator' : 'root'),
        authMethod: formData.auth_method,
        password: formData.type === 'VNC' ? formData.vncPassword : formData.password,
        sshKeyId: formData.selected_ssh_key_id,
        proxyId: formData.type === 'SSH' && advancedConnectionMode.value === 'proxy' ? formData.proxy_id : null,
        jumpChain: formData.type === 'SSH' && advancedConnectionMode.value === 'jump' ? formData.jump_chain : null,
        tagNames,
        notes: formData.notes,
      });

      if (isEditMode.value && connectionId.value) {
        await connectionsApi.update(connectionId.value, payload);
        notify.addNotification('success', '连接已更新');
      } else {
        await createFromPayload(payload, hostResult.ips ?? [formData.host.trim()]);
        notify.addNotification('success', hostResult.ips ? `已创建 ${hostResult.ips.length} 个连接` : '连接已创建');
      }

      await emitSavedAndClose();
    } catch (error) {
      notify.addNotification('error', `保存连接失败: ${getErrorMessage(error)}`);
    } finally {
      isSubmitting.value = false;
    }
  };

  const handleDeleteConnection = async () => {
    if (!isEditMode.value || !connectionId.value) {
      return;
    }
    const confirmed = await confirm('删除连接', `确定删除连接“${formData.name || formData.host}”吗？`);
    if (!confirmed) {
      return;
    }

    try {
      await connectionsApi.delete(connectionId.value);
      notify.addNotification('success', '连接已删除');
      await emitSavedAndClose();
    } catch (error) {
      notify.addNotification('error', `删除连接失败: ${getErrorMessage(error)}`);
    }
  };

  const handleCreateTag = async (tagName: string) => {
    const normalized = tagName.trim();
    if (!normalized) {
      return;
    }

    try {
      const existingTag = tags.value.find(tag => tag.name === normalized);
      if (!existingTag) {
        await tagsStore.create(normalized);
      }
      await tagsStore.fetchAll();
      const createdTag = tagsStore.items.find(tag => tag.name === normalized);
      if (createdTag && !formData.tag_ids.includes(createdTag.id)) {
        formData.tag_ids = [...formData.tag_ids, createdTag.id];
      }
    } catch (error) {
      notify.addNotification('error', `创建标签失败: ${getErrorMessage(error)}`);
    }
  };

  const handleDeleteTag = async (tagId: number) => {
    const tagName = tags.value.find(tag => tag.id === tagId)?.name ?? `ID:${tagId}`;
    const confirmed = await confirm('删除标签', `确定删除标签“${tagName}”吗？`);
    if (!confirmed) {
      return;
    }

    try {
      await tagsStore.remove(tagId);
      formData.tag_ids = formData.tag_ids.filter(id => id !== tagId);
      await tagsStore.fetchAll();
    } catch (error) {
      notify.addNotification('error', `删除标签失败: ${getErrorMessage(error)}`);
    }
  };

  const handleTestConnection = async () => {
    if (formData.type !== 'SSH') {
      return;
    }

    const valid = await validateNormalForm();
    if (!valid) {
      return;
    }

    testStatus.value = 'testing';
    testResult.value = null;
    testLatency.value = null;

    const startedAt = performance.now();
    try {
      let success = false;
      if (isEditMode.value && connectionId.value && formData.auth_method === 'password' && !formData.password.trim()) {
        success = await connectionsApi.test(connectionId.value);
      } else {
        const payload = buildPayload({
          type: 'SSH',
          name: formData.name,
          host: formData.host.trim(),
          port: formData.port,
          username: formData.username.trim(),
          authMethod: formData.auth_method,
          password: formData.password,
          sshKeyId: formData.selected_ssh_key_id,
          proxyId: advancedConnectionMode.value === 'proxy' ? formData.proxy_id : null,
          jumpChain: advancedConnectionMode.value === 'jump' ? formData.jump_chain : null,
          tagNames: mapTagIdsToNames(formData.tag_ids),
          notes: formData.notes,
        });
        success = await connectionsApi.testUnsaved(payload);
      }

      if (success) {
        testStatus.value = 'success';
        testLatency.value = Math.max(1, Math.round(performance.now() - startedAt));
        testResult.value = `${testLatency.value} ms`;
      } else {
        testStatus.value = 'error';
        testResult.value = '测试失败';
      }
    } catch (error) {
      testStatus.value = 'error';
      testResult.value = getErrorMessage(error);
    }
  };

  const addJumpHost = () => {
    if (!formData.jump_chain) {
      formData.jump_chain = [];
    }
    formData.jump_chain.push(null);
  };

  const removeJumpHost = (index: number) => {
    if (!formData.jump_chain) {
      return;
    }
    if (index < 0 || index >= formData.jump_chain.length) {
      return;
    }
    formData.jump_chain.splice(index, 1);
  };

  return {
    formData,
    isLoading,
    testStatus,
    testResult,
    testLatency,
    isScriptModeActive,
    scriptInputText,
    isEditMode,
    formTitle,
    submitButtonText,
    proxies,
    tags,
    connections,
    isProxyLoading,
    proxyStoreError,
    isTagLoading,
    tagStoreError,
    advancedConnectionMode,
    addJumpHost,
    removeJumpHost,
    handleSubmit,
    handleDeleteConnection,
    handleTestConnection,
    handleCreateTag,
    handleDeleteTag,
    latencyColor,
    testButtonText,
    scriptModeFormatInfo,
  };
}

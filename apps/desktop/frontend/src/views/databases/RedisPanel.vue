<template>
  <div class="db-layout">
    <aside class="db-sidebar">
      <div class="section-title">Redis</div>

      <DbConnectionList
        title="连接"
        :items="connectionItems"
        :active-id="activeConnectionId"
        @select="selectConnection"
        @create="openCreate"
        @edit="openEdit"
        @delete="handleDelete"
        @refresh="loadConnections"
      />

      <div class="tree-section">
        <div class="section-title">Keys 扫描</div>
        <div class="field-block">
          <label class="field-label">Key Pattern（可选）</label>
          <input v-model="scanPattern" class="field-input" type="text" placeholder="例如：user:*（留空表示全部）" />
        </div>
        <div class="field-block">
          <label class="field-label">Scan Limit</label>
          <input v-model.number="scanLimit" class="field-input" type="number" min="1" max="20000" />
        </div>
      </div>

      <div v-if="activeConnection" class="tree-section">
        <div class="section-title row">
          <span>资产树（Keys）</span>
          <button type="button" class="btn btn-ghost btn-sm" :disabled="keysLoading" @click="scanKeys">刷新</button>
        </div>

        <div v-if="keysLoading" class="empty">加载中...</div>
        <div v-else-if="keys.length === 0" class="empty">暂无 Key</div>
        <div v-else class="table-list">
          <button
            v-for="k in keys"
            :key="k.key"
            type="button"
            class="table-item"
            :class="{ active: k.key === activeKey }"
            :title="k.key"
            @click="selectKey(k.key)"
          >
            <span class="table-name">{{ k.key }}</span>
            <span class="table-kind">{{ k.keyType }}</span>
          </button>
        </div>
      </div>
    </aside>

    <section class="db-main">
      <RedisMainPanel
        :has-connection="!!activeConnection"
        v-model:password="activePassword"
        v-model:db="activeDb"
        v-model:command="commandText"
        :command-loading="commandLoading"
        :command-error="commandError"
        :command-result="commandResult"
        :active-key="activeKey"
        :key-detail="keyDetail"
        :key-loading="keyLoading"
        @applyDbSwitch="applyDbSwitch"
        @runCommand="runCommand"
        @clearCommand="clearCommand"
        @insertGet="insertGet"
      />
    </section>
  </div>

  <RedisConnectionDialog
    :visible="dialogVisible"
    :title="dialogTitle"
    :save-label="dialogSaveLabel"
    :mode="dialogMode"
    :busy="dialogBusy"
    :error="dialogError"
    v-model:name="formName"
    v-model:host="formHost"
    v-model:port="formPort"
    v-model:username="formUsername"
    v-model:password="formPassword"
    v-model:db="formDb"
    v-model:clearSavedPassword="clearSavedPassword"
    @close="closeDialog"
    @save="saveDialog"
  />
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';

import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { cryptoApi } from '@/lib/api';
import { type ConnectionAsset, type RedisConn, useRedisManager } from './useRedisManager';
import DbConnectionList, { type DbConnListItem } from './components/DbConnectionList.vue';
import RedisConnectionDialog from './components/RedisConnectionDialog.vue';
import RedisMainPanel from './components/RedisMainPanel.vue';

const formName = ref('');
const formHost = ref('127.0.0.1');
const formPort = ref<number>(6379);
const formUsername = ref('');
const formPassword = ref('');
const formDb = ref<number | null>(0);

const {
  connections,
  activeConnectionId,
  activeConnection,
  activeDb,
  activePassword,
  scanPattern,
  scanLimit,
  keys,
  keysLoading,
  activeKey,
  keyDetail,
  keyLoading,
  commandText,
  commandLoading,
  commandError,
  commandResult,
  loadConnections,
  addConnection,
  updateConnection,
  selectConnection,
  removeConnection,
  scanKeys,
  selectKey,
  insertGet,
  clearCommand,
  runCommand,
} = useRedisManager();

const { confirm } = useConfirmDialog();

const connectionItems = computed<DbConnListItem[]>(() =>
  connections.value.map((asset) => ({
    id: asset.id,
    name: asset.name,
    subtitle: describeConn(asset.conn),
  })),
);

function describeConn(conn: RedisConn): string {
  const user = conn.username ? `${conn.username}@` : '';
  const db = typeof conn.db === 'number' ? `/${conn.db}` : '';
  return `${user}${conn.host}:${conn.port}${db}`;
}

function makeDefaultName(conn: RedisConn): string {
  return `Redis · ${conn.host}:${conn.port}`;
}

async function applyDbSwitch() {
  activeKey.value = '';
  keyDetail.value = null;
  await scanKeys();
}

const dialogVisible = ref(false);
const dialogMode = ref<'create' | 'edit'>('create');
const dialogBusy = ref(false);
const dialogError = ref('');
const editingId = ref<string | null>(null);
const editingEncryptedPassword = ref<string | undefined>(undefined);
const clearSavedPassword = ref(false);

const dialogTitle = computed(() => (dialogMode.value === 'create' ? '新建连接（Redis）' : '编辑连接（Redis）'));
const dialogSaveLabel = computed(() => (dialogMode.value === 'create' ? '创建' : '保存'));

function resetForm() {
  formName.value = '';
  formHost.value = '127.0.0.1';
  formPort.value = 6379;
  formUsername.value = '';
  formPassword.value = '';
  formDb.value = 0;
}

function openCreate() {
  dialogMode.value = 'create';
  editingId.value = null;
  dialogError.value = '';
  editingEncryptedPassword.value = undefined;
  clearSavedPassword.value = false;
  resetForm();
  dialogVisible.value = true;
}

function openEdit(id: string) {
  const asset = connections.value.find(c => c.id === id);
  if (!asset) {
    return;
  }
  dialogMode.value = 'edit';
  editingId.value = id;
  dialogError.value = '';
  editingEncryptedPassword.value = asset.encryptedPassword;
  clearSavedPassword.value = false;
  formName.value = asset.name;
  formHost.value = asset.conn.host;
  formPort.value = asset.conn.port;
  formUsername.value = asset.conn.username ?? '';
  formDb.value = typeof asset.conn.db === 'number' ? asset.conn.db : 0;
  formPassword.value = '';
  dialogVisible.value = true;
}

function closeDialog() {
  dialogVisible.value = false;
}

async function saveDialog() {
  const host = formHost.value.trim();
  if (!host) {
    dialogError.value = '请填写 Host';
    return;
  }

  const port = Number(formPort.value) || 6379;
  const username = formUsername.value.trim();
  const db = formDb.value;

  const conn: RedisConn = {
    host,
    port,
    username: username || undefined,
    db: typeof db === 'number' && db >= 0 ? Math.round(db) : undefined,
  };

  const asset: ConnectionAsset<RedisConn> = {
    id: editingId.value ?? crypto.randomUUID(),
    name: formName.value.trim() || makeDefaultName(conn),
    conn,
  };

  dialogBusy.value = true;
  dialogError.value = '';
  try {
    const password = formPassword.value.trim();

    let encryptedPassword = dialogMode.value === 'edit' ? editingEncryptedPassword.value : undefined;
    if (dialogMode.value === 'edit' && clearSavedPassword.value) {
      encryptedPassword = undefined;
    }
    if (password) {
      encryptedPassword = await cryptoApi.encrypt(password);
    }
    asset.encryptedPassword = encryptedPassword;

    if (dialogMode.value === 'create') {
      await addConnection(asset, '');
    } else {
      await updateConnection(asset);
    }
    dialogVisible.value = false;
  } catch (e: unknown) {
    dialogError.value = e instanceof Error ? e.message : '保存失败';
  } finally {
    dialogBusy.value = false;
  }
}

async function handleDelete(id: string) {
  const asset = connections.value.find(c => c.id === id);
  if (!asset) {
    return;
  }
  const ok = await confirm('删除连接', `确认删除 “${asset.name}” 吗？`);
  if (!ok) {
    return;
  }
  await removeConnection(id);
}

onMounted(() => {
  void loadConnections();
});
</script>

<template>
  <div class="connections-page">
    <div class="connections-container">
      <h1 class="page-title">连接管理</h1>

      <div class="connections-card">
        <!-- Toolbar -->
        <div class="toolbar">
          <h2 class="toolbar-title">连接列表 ({{ filteredList.length }})</h2>
          <div class="toolbar-controls">
            <input
              v-model="searchQuery"
              type="text"
              placeholder="搜索连接..."
              class="search-input"
            />
            <select v-model="filterTag" class="filter-select">
              <option value="">所有标签</option>
              <option v-for="t in allTags" :key="t.id" :value="t.name">{{ t.name }}</option>
            </select>
            <select v-model="sortBy" class="filter-select">
              <option value="name">名称</option>
              <option value="host">主机</option>
            </select>
            <button class="icon-btn" @click="sortAsc = !sortAsc" :title="sortAsc ? '升序' : '降序'">
              {{ sortAsc ? '↑' : '↓' }}
            </button>
            <button class="btn-primary" @click="openCreate" title="新建连接">+</button>
          </div>
        </div>

        <!-- Connection List -->
        <div class="list-body">
          <div v-if="loading && !filteredList.length" class="empty-state">加载中...</div>
          <ul v-else-if="filteredList.length" class="conn-list">
            <li v-for="c in filteredList" :key="c.id" class="conn-item">
              <div class="conn-info">
                <span class="conn-name">{{ c.name || c.host }}</span>
                <span class="conn-detail">{{ (c.type || 'SSH') }} · {{ c.username ? c.username + '@' : '' }}{{ c.host }}:{{ c.port }}</span>
                <div v-if="c.tags?.length" class="conn-tags">
                  <span v-for="tag in c.tags" :key="tag" class="tag-badge">{{ tag }}</span>
                </div>
              </div>
              <div class="conn-actions">
                <button class="btn-outline" @click.stop="openEdit(c.id)">编辑</button>
                <button class="btn-outline" @click.stop="handleClone(c.id)">克隆</button>
                <button class="btn-outline btn-danger-outline" @click.stop="handleDelete(c)">删除</button>
                <button class="btn-connect" @click.stop="handleConnect(c)">连接</button>
              </div>
            </li>
          </ul>
          <div v-else class="empty-state">暂无连接</div>
        </div>
      </div>
    </div>

    <AddConnectionForm
      :visible="formVisible"
      :mode="formMode"
      :connection-id="editId"
      @close="formVisible = false"
      @saved="onSaved"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useConnectionsStore } from '@/stores/connections';
import { useSessionStore } from '@/stores/session';
import { useConfirmDialog } from '@/composables/useConfirmDialog';
import { storeToRefs } from 'pinia';
import { connectionsApi, sshApi, sftpApi, desktopApi, type Connection } from '@/lib/api';
import AddConnectionForm from '@/components/AddConnectionForm.vue';

const store = useConnectionsStore();
const sessionStore = useSessionStore();
const { list, tags: allTags, loading } = storeToRefs(store);
const { confirm } = useConfirmDialog();
const router = useRouter();

const formVisible = ref(false);
const formMode = ref<'create' | 'edit'>('create');
const editId = ref<number>();

const searchQuery = ref('');
const filterTag = ref('');
const sortBy = ref<'name' | 'host'>('name');
const sortAsc = ref(true);

const filteredList = computed(() => {
  let result = [...list.value];
  const q = searchQuery.value.toLowerCase().trim();
  if (q) {
    result = result.filter(c =>
      c.name?.toLowerCase().includes(q) ||
      c.host?.toLowerCase().includes(q) ||
      c.username?.toLowerCase().includes(q) ||
      String(c.port).includes(q)
    );
  }
  if (filterTag.value) {
    result = result.filter(c => c.tags?.includes(filterTag.value));
  }
  const factor = sortAsc.value ? 1 : -1;
  result.sort((a, b) => {
    const va = (a[sortBy.value] || '') as string;
    const vb = (b[sortBy.value] || '') as string;
    return va.localeCompare(vb) * factor;
  });
  return result;
});

function openCreate() { formMode.value = 'create'; editId.value = undefined; formVisible.value = true; }
function openEdit(id: number) { formMode.value = 'edit'; editId.value = id; formVisible.value = true; }
async function onSaved() { formVisible.value = false; await store.fetch(); }
async function handleClone(id: number) { await connectionsApi.clone(id); await store.fetch(); }
async function handleDelete(c: Connection) {
  if (await confirm('删除连接', `确定删除 "${c.name}" 吗？`)) { await store.remove(c.id); }
}

async function warmupSftp(sessionId: string, connectionId: number) {
  try {
    const sftpSessionId = await sftpApi.open(connectionId);
    sessionStore.setSftpSession(sessionId, sftpSessionId);
  } catch {
    // ignore sftp warmup failures; terminal session can still work
  }
}

async function handleConnect(c: Connection) {
  if (String(c.type ?? 'SSH').toUpperCase() === 'RDP') {
    try {
      await desktopApi.openRdpConnection(c.id);
    } catch (e: any) {
      window.alert(`RDP 启动失败: ${e.message ?? String(e)}`);
    }
    return;
  }

  const sid = sessionStore.createSession(c.id, c.name);
  sessionStore.setActive(sid);
  void router.push('/workspace');

  try {
    const realSid = await sshApi.connect(c.id);
    sessionStore.removeSession(sid);
    sessionStore.addSession({
      id: realSid,
      connectionId: c.id,
      connectionName: c.name,
      status: 'connected',
      createdAt: new Date().toISOString(),
      sftpReady: false,
      sftpSessionId: null,
      currentPath: '/',
    });
    sessionStore.setActive(realSid);
    void warmupSftp(realSid, c.id);
  } catch {
    sessionStore.updateStatus(sid, 'disconnected');
  }
}
onMounted(() => store.fetch());
</script>

<style scoped>
.connections-page { padding: 20px 24px; height: 100%; overflow-y: auto; }
.connections-container { max-width: 960px; margin: 0 auto; }
.page-title { font-size: 20px; font-weight: 600; color: var(--text); margin-bottom: 16px; }

.connections-card {
  background: var(--bg-surface0);
  border: 1px solid var(--border);
  border-radius: 8px;
  overflow: hidden;
  min-height: 400px;
}

.toolbar {
  display: flex; justify-content: space-between; align-items: center;
  padding: 12px 16px; border-bottom: 1px solid var(--border);
  flex-wrap: wrap; gap: 8px;
}
.toolbar-title { font-size: 15px; font-weight: 500; color: var(--text); white-space: nowrap; }
.toolbar-controls { display: flex; align-items: center; gap: 8px; flex-wrap: wrap; }

.search-input {
  height: 32px; padding: 0 10px; font-size: 13px;
  border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg-base); color: var(--text);
  outline: none; width: 180px;
}
.search-input:focus { border-color: var(--blue); }

.filter-select {
  height: 32px; padding: 0 8px; font-size: 13px;
  border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg-base); color: var(--text);
  outline: none; cursor: pointer;
}

.icon-btn {
  height: 32px; width: 32px; display: flex; align-items: center; justify-content: center;
  border: 1px solid var(--border); border-radius: 4px;
  background: var(--bg-base); color: var(--text); cursor: pointer; font-size: 14px;
}
.icon-btn:hover { background: var(--bg-surface1); }

.btn-primary {
  height: 32px; width: 32px; display: flex; align-items: center; justify-content: center;
  background: var(--blue); color: var(--bg-base); border: none; border-radius: 4px;
  cursor: pointer; font-size: 16px; font-weight: 600;
}
.btn-primary:hover { opacity: 0.9; }

.list-body { padding: 12px 16px; }
.conn-list { list-style: none; padding: 0; margin: 0; display: flex; flex-direction: column; gap: 10px; }

.conn-item {
  display: flex; align-items: center; justify-content: space-between;
  padding: 12px 14px; border: 1px solid var(--border); border-radius: 6px;
  background: var(--bg-surface1); transition: background 0.15s;
}
.conn-item:hover { background: var(--bg-surface0); }

.conn-info { flex: 1; min-width: 0; margin-right: 12px; }
.conn-name { display: block; font-size: 14px; font-weight: 500; color: var(--text); }
.conn-detail { display: block; font-size: 12px; color: var(--text-sub); margin-top: 2px; }
.conn-tags { display: flex; flex-wrap: wrap; gap: 4px; margin-top: 6px; }
.tag-badge {
  padding: 1px 8px; font-size: 11px; border-radius: 3px;
  background: var(--bg-base); color: var(--text-dim); border: 1px solid var(--border);
}

.conn-actions { display: flex; align-items: center; gap: 6px; flex-shrink: 0; }
.btn-outline {
  padding: 5px 12px; font-size: 12px; border-radius: 4px;
  border: 1px solid var(--border); background: transparent; color: var(--text);
  cursor: pointer; white-space: nowrap;
}
.btn-outline:hover { background: var(--bg-surface0); }
.btn-danger-outline { color: var(--red); border-color: var(--red); }
.btn-danger-outline:hover { background: rgba(243,139,168,0.1); }

.btn-connect {
  padding: 6px 16px; font-size: 12px; font-weight: 500; border-radius: 4px;
  background: var(--blue); color: var(--bg-base); border: none;
  cursor: pointer; white-space: nowrap;
}
.btn-connect:hover { opacity: 0.9; }

.empty-state { text-align: center; color: var(--text-dim); padding: 40px; font-size: 14px; }
</style>






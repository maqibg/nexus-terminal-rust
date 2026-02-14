<template>
  <div class="layout">
    <aside class="sidebar">
      <div class="sidebar-header">
        <h2>连接</h2>
        <button class="btn-icon" @click="dialogConn = undefined; showDialog = true" title="新建连接">+</button>
      </div>
      <div class="search-bar">
        <input class="search-input" v-model="searchQuery" placeholder="搜索连接..." />
      </div>
      <div class="tag-filter" v-if="connStore.tags.length">
        <button v-for="tag in connStore.tags" :key="tag.id" class="tag-btn" :class="{ active: filterTag === tag.name }" @click="filterTag = filterTag === tag.name ? '' : tag.name">{{ tag.name }}</button>
      </div>
      <div v-if="connStore.loading" class="loading">加载中...</div>
      <div v-else class="conn-list">
        <div
          v-for="conn in filteredConnections" :key="conn.id"
          class="conn-item" :class="{ active: activeConnId === conn.id }"
          @click="openSession(conn)"
          @contextmenu.prevent="showCtx($event, conn)"
        >
          <span class="conn-name">{{ conn.name }}</span>
          <span class="conn-host">{{ conn.host }}:{{ conn.port }}</span>
          <div class="conn-tags" v-if="connStore.getTagsForConnection(conn).length">
            <span v-for="tag in connStore.getTagsForConnection(conn)" :key="tag.id" class="tag-badge" :title="tag.name">{{ tag.name }}</span>
          </div>
        </div>
        <div v-if="!connStore.list.length" class="empty">暂无连接</div>
      </div>
      <div class="sidebar-qc">
        <QuickCommands @execute="sendQuickCommand" />
      </div>
      <div class="sidebar-history">
        <CommandHistory @execute="sendQuickCommand" />
      </div>
      <div class="sidebar-footer">
        <button class="btn-text" @click="showSuspendedModal = true">挂起</button>
        <button class="btn-text" @click="$router.push('/settings')">设置</button>
        <button class="btn-text" @click="$router.push('/audit')">日志</button>
        <button class="btn-text" @click="handleLogout">退出</button>
      </div>
    </aside>
    <main class="content">
      <div v-if="!tabs.length" class="empty-state">
        <p>选择左侧连接开始 SSH 会话</p>
      </div>
      <div v-else class="tabs-area">
        <div class="tab-bar">
          <div
            v-for="tab in tabs" :key="tab.sessionId"
            class="tab" :class="{ active: activeTab === tab.sessionId }"
            @click="activeTab = tab.sessionId"
            @contextmenu.prevent="showTabCtx($event, tab.sessionId)"
          >
            <span>{{ tab.name }}</span>
            <button class="tab-close" @click.stop="closeTab(tab.sessionId)">×</button>
          </div>
        </div>
        <!-- Panel toggle -->
        <div class="panel-toggle" v-if="activeTab">
          <button :class="{ active: activePanel === 'terminal' }" @click="activePanel = 'terminal'">终端</button>
          <button :class="{ active: activePanel === 'sftp' }" @click="openSftp">文件</button>
        </div>
        <div class="panel-area">
          <div v-show="activePanel === 'terminal'" class="terminal-area">
            <TerminalView
              v-for="tab in tabs" :key="tab.sessionId"
              v-show="activeTab === tab.sessionId"
              :session-id="tab.sessionId"
            />
          </div>
          <div v-if="activePanel === 'sftp' && activeSftpId" class="sftp-area">
            <SftpBrowser :session-id="activeSftpId" :connection-id="activeConnectionId" />
          </div>
          <div v-if="activePanel === 'sftp' && sftpLoading" class="sftp-loading">SFTP 连接中...</div>
        </div>
      </div>
    </main>

    <!-- Context menu -->
    <div v-if="ctxConn" class="ctx-menu" :style="{ left: ctxPos.x + 'px', top: ctxPos.y + 'px' }" @click="ctxConn = null">
      <div class="ctx-item" @click="openSession(ctxConn!)">连接</div>
      <div class="ctx-item" @click="dialogConn = ctxConn!; showDialog = true">编辑</div>
      <div class="ctx-item" @click="handleClone(ctxConn!)">克隆</div>
      <div class="ctx-item" @click="handleTest(ctxConn!)">测试</div>
      <div class="ctx-item danger" @click="handleDelete(ctxConn!)">删除</div>
    </div>
    <div v-if="ctxConn" class="ctx-backdrop" @click="ctxConn = null"></div>

    <!-- Connection dialog -->
    <ConnectionDialog v-if="showDialog" :connection="dialogConn" @close="showDialog = false" @saved="connStore.fetch()" />

    <!-- Tab context menu -->
    <TabBarContextMenu :visible="tabCtx.visible" :x="tabCtx.x" :y="tabCtx.y" :session-id="tabCtx.sessionId" @close="tabCtx.visible = false" @action="handleTabAction" />

    <!-- Modals -->
    <SuspendedSshSessionsModal :visible="showSuspendedModal" @close="showSuspendedModal = false" />
    <BatchEditConnectionForm :visible="showBatchEdit" :connection-ids="batchEditIds" @saved="showBatchEdit = false; connStore.fetch()" @cancel="showBatchEdit = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';
import { useConnectionsStore } from '@/stores/connections';
import { sshApi, sftpApi, connectionsApi, desktopApi } from '@/lib/api';
import type { Connection } from '@/lib/api';
import TerminalView from '@/components/TerminalView.vue';
import ConnectionDialog from '@/components/ConnectionDialog.vue';
import SftpBrowser from '@/components/SftpBrowser.vue';
import QuickCommands from '@/components/QuickCommands.vue';
import CommandHistory from '@/components/CommandHistory.vue';
import TabBarContextMenu from '@/components/TabBarContextMenu.vue';
import SuspendedSshSessionsModal from '@/components/SuspendedSshSessionsModal.vue';
import BatchEditConnectionForm from '@/components/BatchEditConnectionForm.vue';

const auth = useAuthStore();
const connStore = useConnectionsStore();
const router = useRouter();

const showDialog = ref(false);
const dialogConn = ref<Connection | undefined>();
const activeConnId = ref<number | null>(null);
const activeTab = ref('');
const activePanel = ref<'terminal' | 'sftp'>('terminal');
const searchQuery = ref('');
const filterTag = ref('');

// Modal states
const showSuspendedModal = ref(false);
const showBatchEdit = ref(false);
const batchEditIds = ref<number[]>([]);

// Tab context menu
const tabCtx = ref({ visible: false, x: 0, y: 0, sessionId: '' });

function showTabCtx(e: MouseEvent, sessionId: string) {
  e.preventDefault();
  tabCtx.value = { visible: true, x: e.clientX, y: e.clientY, sessionId };
}

function handleTabAction(action: string) {
  const sid = tabCtx.value.sessionId;
  tabCtx.value.visible = false;
  if (action === 'close') { closeTab(sid); }
  else if (action === 'close-others') { tabs.value.filter(t => t.sessionId !== sid).forEach(t => closeTab(t.sessionId)); }
  else if (action === 'close-right') {
    const idx = tabs.value.findIndex(t => t.sessionId === sid);
    tabs.value.slice(idx + 1).forEach(t => closeTab(t.sessionId));
  } else if (action === 'duplicate') {
    const tab = tabs.value.find(t => t.sessionId === sid);
    if (tab) { const conn = connStore.list.find(c => c.id === tab.connectionId); if (conn) openSession(conn); }
  }
}

const filteredConnections = computed(() => {
  let list = connStore.list;
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase();
    list = list.filter(c => c.name.toLowerCase().includes(q) || c.host.toLowerCase().includes(q));
  }
  if (filterTag.value) {
    list = list.filter(c => c.tags.includes(filterTag.value));
  }
  return list;
});

// Context menu
const ctxConn = ref<Connection | null>(null);
const ctxPos = ref({ x: 0, y: 0 });

function showCtx(e: MouseEvent, conn: Connection) {
  ctxConn.value = conn;
  ctxPos.value = { x: e.clientX, y: e.clientY };
}

async function handleDelete(conn: Connection) {
  if (!confirm(`确定删除连接 "${conn.name}"？`)) return;
  await connStore.remove(conn.id);
}

async function handleClone(conn: Connection) {
  try {
    await connectionsApi.clone(conn.id);
    connStore.fetch();
  } catch (e: any) { alert(`克隆失败: ${e.message}`); }
}

async function handleTest(conn: Connection) {
  try {
    const ok = await connectionsApi.test(conn.id);
    alert(ok ? `连接 "${conn.name}" 测试成功` : `连接 "${conn.name}" 测试失败`);
  } catch (e: any) { alert(`测试失败: ${e.message}`); }
}

interface Tab { sessionId: string; name: string; connectionId: number; sftpId?: string; }
const tabs = ref<Tab[]>([]);
const sftpLoading = ref(false);

const activeSftpId = ref<string | null>(null);

const activeConnectionId = computed(() => {
  const tab = tabs.value.find(t => t.sessionId === activeTab.value);
  return tab?.connectionId;
});

onMounted(() => { connStore.fetch(); });

async function openSession(conn: Connection) {
  activeConnId.value = conn.id;

  if (String(conn.type ?? 'SSH').toUpperCase() === 'RDP') {
    try {
      await desktopApi.openRdpConnection(conn.id);
    } catch (e: any) {
      alert(`RDP 启动失败: ${e.message}`);
    }
    return;
  }

  try {
    const sessionId = await sshApi.connect(conn.id, 120, 36);
    tabs.value.push({ sessionId, name: conn.name, connectionId: conn.id });
    activeTab.value = sessionId;
    activePanel.value = 'terminal';
  } catch (e: any) {
    alert(`连接失败: ${e.message}`);
  }
}
async function openSftp() {
  activePanel.value = 'sftp';
  const tab = tabs.value.find(t => t.sessionId === activeTab.value);
  if (!tab) return;

  // 已有 SFTP session
  if (tab.sftpId) {
    activeSftpId.value = tab.sftpId;
    return;
  }

  sftpLoading.value = true;
  activeSftpId.value = null;
  try {
    const sftpId = await sftpApi.open(tab.connectionId);
    tab.sftpId = sftpId;
    activeSftpId.value = sftpId;
  } catch (e: any) {
    alert(`SFTP 连接失败: ${e.message}`);
    activePanel.value = 'terminal';
  } finally {
    sftpLoading.value = false;
  }
}

async function closeTab(sessionId: string) {
  const tab = tabs.value.find(t => t.sessionId === sessionId);
  if (tab?.sftpId) {
    sftpApi.close(tab.sftpId).catch(() => {});
  }
  try { await sshApi.close(sessionId); } catch { /* ignore */ }
  tabs.value = tabs.value.filter(t => t.sessionId !== sessionId);
  if (activeTab.value === sessionId) {
    activeTab.value = tabs.value[0]?.sessionId ?? '';
    activeSftpId.value = null;
    activePanel.value = 'terminal';
  }
}

async function handleLogout() {
  for (const tab of tabs.value) {
    if (tab.sftpId) sftpApi.close(tab.sftpId).catch(() => {});
    try { await sshApi.close(tab.sessionId); } catch { /* ignore */ }
  }
  tabs.value = [];
  await auth.logout();
  router.push('/login');
}

function sendQuickCommand(command: string) {
  if (!activeTab.value) return;
  const b64 = btoa(command + '\n');
  sshApi.write(activeTab.value, b64).catch(() => {});
}
</script>

<style scoped>
.layout { display: flex; height: 100%; }
.sidebar {
  width: 240px; min-width: 240px; background: #181825;
  display: flex; flex-direction: column; border-right: 1px solid #313244;
}
.sidebar-header {
  display: flex; align-items: center; justify-content: space-between;
  padding: 0.75rem 1rem; border-bottom: 1px solid #313244;
}
.sidebar-header h2 { font-size: 1rem; font-weight: 500; margin: 0; color: #cdd6f4; }
.btn-icon {
  width: 28px; height: 28px; border-radius: 6px; border: 1px solid #45475a;
  background: transparent; color: #cdd6f4; cursor: pointer; font-size: 1.1rem;
  display: flex; align-items: center; justify-content: center;
}
.btn-icon:hover { background: #313244; }
.conn-list { flex: 1; overflow-y: auto; padding: 0.5rem; }
.conn-item {
  padding: 0.5rem 0.75rem; border-radius: 6px; cursor: pointer;
  display: flex; flex-direction: column; gap: 2px; margin-bottom: 2px;
}
.conn-item:hover { background: #313244; }
.conn-item.active { background: #45475a; }
.conn-name { font-size: 0.85rem; color: #cdd6f4; }
.conn-host { font-size: 0.75rem; color: #6c7086; }
.conn-tags { display: flex; gap: 3px; margin-top: 2px; }
.tag-badge {
  font-size: 0.6rem; color: #89b4fa; background: rgba(137,180,250,0.1);
  padding: 1px 4px; border-radius: 3px;
}
.sidebar-footer { padding: 0.5rem 1rem; border-top: 1px solid #313244; display: flex; gap: 12px; }
.sidebar-qc { height: 200px; border-top: 1px solid #313244; overflow: hidden; }
.sidebar-history { height: 160px; border-top: 1px solid #313244; overflow: hidden; }
.btn-text {
  background: none; border: none; color: #a6adc8; cursor: pointer;
  font-size: 0.8rem; padding: 0.25rem 0;
}
.btn-text:hover { color: #f38ba8; }
.content { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.empty-state {
  flex: 1; display: flex; align-items: center; justify-content: center;
  color: #6c7086; font-size: 0.9rem;
}
.tabs-area { flex: 1; display: flex; flex-direction: column; overflow: hidden; }
.tab-bar {
  display: flex; background: #181825; border-bottom: 1px solid #313244;
  overflow-x: auto; min-height: 36px;
}
.tab {
  display: flex; align-items: center; gap: 0.5rem;
  padding: 0 1rem; cursor: pointer; white-space: nowrap;
  font-size: 0.8rem; color: #a6adc8; border-right: 1px solid #313244;
}
.tab:hover { background: #313244; }
.tab.active { background: #1e1e2e; color: #cdd6f4; }
.tab-close {
  background: none; border: none; color: inherit; cursor: pointer;
  font-size: 1rem; padding: 0; line-height: 1;
}
.tab-close:hover { color: #f38ba8; }

.panel-toggle {
  display: flex; background: #181825; border-bottom: 1px solid #313244;
  padding: 0 8px;
}
.panel-toggle button {
  padding: 4px 12px; font-size: 0.75rem; cursor: pointer;
  background: transparent; border: none; border-bottom: 2px solid transparent;
  color: #6c7086;
}
.panel-toggle button.active { color: #89b4fa; border-bottom-color: #89b4fa; }
.panel-toggle button:hover { color: #cdd6f4; }

.panel-area { flex: 1; overflow: hidden; position: relative; }
.terminal-area { width: 100%; height: 100%; position: absolute; inset: 0; }
.sftp-area { width: 100%; height: 100%; position: absolute; inset: 0; }
.sftp-loading {
  position: absolute; inset: 0; display: flex; align-items: center; justify-content: center;
  color: #6c7086; font-size: 0.85rem;
}
.loading, .empty { color: #6c7086; font-size: 0.8rem; padding: 1rem; text-align: center; }

.search-bar { padding: 6px 8px; border-bottom: 1px solid #313244; }
.search-input { width: 100%; padding: 4px 8px; border-radius: 4px; border: 1px solid #45475a; background: #1e1e2e; color: #cdd6f4; font-size: 0.8rem; outline: none; box-sizing: border-box; }
.search-input:focus { border-color: #89b4fa; }
.tag-filter { display: flex; flex-wrap: wrap; gap: 4px; padding: 4px 8px; border-bottom: 1px solid #313244; }
.tag-btn { padding: 1px 6px; border-radius: 3px; border: 1px solid #45475a; background: transparent; color: #a6adc8; cursor: pointer; font-size: 0.65rem; }
.tag-btn:hover { background: #313244; }
.tag-btn.active { background: #89b4fa; color: #1e1e2e; border-color: #89b4fa; }

/* Context menu */
.ctx-backdrop { position: fixed; inset: 0; z-index: 99; }
.ctx-menu {
  position: fixed; z-index: 100; background: #313244; border-radius: 8px;
  padding: 4px; box-shadow: 0 4px 16px rgba(0,0,0,0.4); min-width: 100px;
}
.ctx-item {
  padding: 0.4rem 0.75rem; border-radius: 4px; cursor: pointer;
  font-size: 0.8rem; color: #cdd6f4;
}
.ctx-item:hover { background: #45475a; }
.ctx-item.danger { color: #f38ba8; }
</style>



<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { storeToRefs } from 'pinia';
import { sshApi } from '@/lib/api';
import { useDockerManager } from '@/composables/useDockerManager';
import { useSessionStore } from '@/stores/session';
import { useRouter } from 'vue-router';

const SETTINGS_TAB_STORAGE_KEY = 'settings_active_tab';
const SETTINGS_FOCUS_SECTION_STORAGE_KEY = 'settings_focus_section';

const sessionStore = useSessionStore();
const { activeSession } = storeToRefs(sessionStore);
const router = useRouter();

const activeSshSessionId = computed(() =>
  activeSession.value?.protocol === 'SSH' ? activeSession.value.id : null,
);
const sshConnectionStatus = computed(() =>
  activeSession.value?.protocol === 'SSH' ? activeSession.value.status : 'disconnected',
);

const dockerManager = useDockerManager(activeSshSessionId, sshConnectionStatus);
const containers = computed(() => dockerManager.containers.value);
const isLoading = computed(() => dockerManager.isLoading.value);
const error = computed(() => dockerManager.error.value);
const isDockerAvailable = computed(() => dockerManager.isDockerAvailable.value);
const expandedContainerIds = computed(() => dockerManager.expandedContainerIds.value);
const showSudoPasswordDialog = computed(() => dockerManager.showSudoPasswordDialog.value);
const sudoPasswordError = computed(() => dockerManager.sudoPasswordError.value);
const currentSessionId = computed(() => activeSshSessionId.value);
const sudoPassword = ref('');

watch([showSudoPasswordDialog, currentSessionId], () => {
  sudoPassword.value = '';
});

function sendDockerCommand(containerId: string, command: 'start' | 'stop' | 'restart' | 'remove') {
  void dockerManager.sendDockerCommand(containerId, command);
}

function toggleExpand(containerId: string) {
  dockerManager.toggleExpand(containerId);
}

async function sendTerminalCommand(command: string) {
  if (!currentSessionId.value) {
    return;
  }
  const payload = btoa(unescape(encodeURIComponent(`${command}\n`)));
  await sshApi.write(currentSessionId.value, payload);
}

function enterContainer(containerId: string) {
  void sendTerminalCommand(`docker exec -it ${containerId} sh`);
}

function viewContainerLogs(containerId: string) {
  void sendTerminalCommand(`docker logs --tail 1000 -f ${containerId}`);
}

async function openDockerSettings() {
  localStorage.setItem(SETTINGS_TAB_STORAGE_KEY, 'workspace');
  localStorage.setItem(SETTINGS_FOCUS_SECTION_STORAGE_KEY, 'docker');
  await router.push('/settings#docker-settings');
}

function openSudoPasswordDialog() {
  sudoPassword.value = '';
  dockerManager.requestSudoPassword('');
}

function closeSudoPasswordDialog() {
  sudoPassword.value = '';
  dockerManager.closeSudoPasswordDialog();
}

async function submitSudoPassword() {
  if (!sudoPassword.value.trim()) {
    return;
  }
  const ok = await dockerManager.validateAndStoreSudoPassword(sudoPassword.value);
  if (ok) {
    sudoPassword.value = '';
  }
}
</script>

<template>
  <div class="docker-manager">
    <div class="docker-toolbar">
      <button class="toolbar-btn" title="配置" @click="openDockerSettings">
        <i class="fas fa-cog"></i>
        <span>配置</span>
      </button>
      <button class="toolbar-btn" title="输入 sudo 密码" @click="openSudoPasswordDialog">
        <i class="fas fa-user-shield"></i>
        <span>提权</span>
      </button>
    </div>

    <div v-if="!currentSessionId" class="docker-placeholder">
      <i class="fas fa-plug"></i>
      <p>没有活动 SSH 会话</p>
      <small>请先连接 SSH 会话后再使用 Docker 管理器</small>
    </div>

    <div v-else-if="sshConnectionStatus === 'connecting'" class="docker-placeholder">
      <i class="fas fa-spinner fa-spin"></i>
      <p>等待 SSH 连接...</p>
    </div>

    <div v-else-if="sshConnectionStatus !== 'connected'" class="docker-placeholder">
      <i class="fas fa-unlink"></i>
      <p>SSH 连接不可用</p>
    </div>

    <div v-else-if="isLoading && containers.length === 0" class="docker-placeholder">
      <i class="fas fa-spinner fa-spin"></i>
      <p>加载 Docker 状态...</p>
    </div>

    <div v-else-if="!isDockerAvailable" class="docker-placeholder">
      <i class="fab fa-docker"></i>
      <p>远程主机未安装或无法访问 Docker</p>
      <small>请确保远程主机已安装 Docker 且当前用户有访问权限</small>
      <button class="primary-action-btn" @click="openSudoPasswordDialog">输入 sudo 密码</button>
    </div>

    <div v-else-if="error" class="docker-placeholder error">
      <i class="fas fa-exclamation-triangle"></i>
      <p>获取 Docker 状态失败</p>
      <small>{{ error }}</small>
      <button class="primary-action-btn" @click="openSudoPasswordDialog">输入 sudo 密码</button>
    </div>

    <div v-else class="docker-content-area">
      <div v-if="containers.length === 0 && !isLoading" class="docker-placeholder compact">
        <p>没有容器</p>
      </div>

      <table v-else class="docker-table">
        <thead class="responsive-thead">
          <tr>
            <th class="w-expand"></th>
            <th>名称</th>
            <th>镜像</th>
            <th>状态</th>
            <th>端口</th>
            <th>操作</th>
          </tr>
        </thead>
        <tbody class="responsive-tbody">
          <template v-for="container in containers" :key="container.id">
            <tr class="responsive-tr" :class="{ expanded: expandedContainerIds.has(container.id) }">
              <td class="responsive-td-expand">
                <button class="icon-btn" :title="expandedContainerIds.has(container.id) ? '收起' : '展开'" @click="toggleExpand(container.id)">
                  <i :class="['fas', expandedContainerIds.has(container.id) ? 'fa-chevron-down' : 'fa-chevron-right']"></i>
                </button>
              </td>
              <td class="responsive-td" data-label="名称">
                <span class="font-strong">{{ container.Names?.join(', ') || 'N/A' }}</span>
              </td>
              <td class="responsive-td break-all" data-label="镜像">{{ container.Image }}</td>
              <td class="responsive-td" data-label="状态">
                <span class="status-badge" :class="`state-${container.State}`">{{ container.Status }}</span>
              </td>
              <td class="responsive-td break-all" data-label="端口">
                {{ container.Ports?.map(p => `${p.IP ? `${p.IP}:` : ''}${p.PublicPort ? `${p.PublicPort}->` : ''}${p.PrivatePort}/${p.Type}`).join(', ') || 'N/A' }}
              </td>
              <td class="responsive-td" data-label="操作">
                <div class="action-group">
                  <button class="icon-btn" title="启动" :disabled="container.State === 'running'" @click="sendDockerCommand(container.id, 'start')"><i class="fas fa-play"></i></button>
                  <button class="icon-btn" title="停止" :disabled="container.State !== 'running'" @click="sendDockerCommand(container.id, 'stop')"><i class="fas fa-stop"></i></button>
                  <button class="icon-btn" title="重启" :disabled="container.State !== 'running'" @click="sendDockerCommand(container.id, 'restart')"><i class="fas fa-sync-alt"></i></button>
                  <button class="icon-btn danger" title="删除" @click="sendDockerCommand(container.id, 'remove')"><i class="fas fa-trash-alt"></i></button>
                  <button class="icon-btn" title="进入容器" @click="enterContainer(container.id)"><i class="fas fa-terminal"></i></button>
                  <button class="icon-btn" title="查看日志" @click="viewContainerLogs(container.id)"><i class="fas fa-file-alt"></i></button>
                </div>
              </td>
              <td class="responsive-td-card-expand">
                <div v-if="!expandedContainerIds.has(container.id)">
                  <button class="card-expand-btn" @click="toggleExpand(container.id)">
                    <i class="fas fa-chevron-down"></i> 展开
                  </button>
                </div>
                <div v-else class="card-expand-content">
                  <div class="stats-box">
                    <dl v-if="container.stats" class="stats-grid">
                      <dt>CPU</dt><dd>{{ container.stats.CPUPerc ?? 'N/A' }}</dd>
                      <dt>内存</dt><dd>{{ container.stats.MemUsage ?? 'N/A' }} ({{ container.stats.MemPerc ?? 'N/A' }})</dd>
                      <dt>网络</dt><dd>{{ container.stats.NetIO ?? 'N/A' }}</dd>
                      <dt>磁盘</dt><dd>{{ container.stats.BlockIO ?? 'N/A' }}</dd>
                      <dt>PIDs</dt><dd>{{ container.stats.PIDs ?? 'N/A' }}</dd>
                    </dl>
                    <div v-else class="stats-empty">暂无统计数据</div>
                  </div>
                  <button class="card-expand-btn collapse" @click="toggleExpand(container.id)">
                    <i class="fas fa-chevron-up"></i> 收起
                  </button>
                </div>
              </td>
            </tr>

            <tr v-if="expandedContainerIds.has(container.id)" class="responsive-expansion-row">
              <td colspan="6" class="expansion-cell">
                <div class="stats-box">
                  <dl v-if="container.stats" class="stats-grid">
                    <dt>CPU</dt><dd>{{ container.stats.CPUPerc ?? 'N/A' }}</dd>
                    <dt>内存</dt><dd>{{ container.stats.MemUsage ?? 'N/A' }} ({{ container.stats.MemPerc ?? 'N/A' }})</dd>
                    <dt>网络</dt><dd>{{ container.stats.NetIO ?? 'N/A' }}</dd>
                    <dt>磁盘</dt><dd>{{ container.stats.BlockIO ?? 'N/A' }}</dd>
                    <dt>PIDs</dt><dd>{{ container.stats.PIDs ?? 'N/A' }}</dd>
                  </dl>
                  <div v-else class="stats-empty">暂无统计数据</div>
                </div>
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>

    <div v-if="showSudoPasswordDialog" class="dialog-backdrop" @click.self="closeSudoPasswordDialog">
      <div class="sudo-dialog">
        <div class="sudo-dialog-title">
          <i class="fas fa-user-shield"></i>
          <span>输入 sudo 密码</span>
        </div>
        <input
          v-model="sudoPassword"
          class="sudo-dialog-input"
          type="password"
          placeholder="输入当前账户的 sudo 密码..."
          @keydown.enter.prevent="submitSudoPassword"
        >
        <div v-if="sudoPasswordError" class="sudo-dialog-error">{{ sudoPasswordError }}</div>
        <div class="sudo-dialog-hint">密码仅保存在当前桌面会话内存中，不会写入数据库。</div>
        <div class="sudo-dialog-actions">
          <button class="secondary-action-btn" @click="closeSudoPasswordDialog">取消</button>
          <button class="primary-action-btn" @click="submitSudoPassword">确认</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.docker-manager {
  container-type: inline-size;
  container-name: docker-manager-pane;
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
}
.docker-toolbar {
  display: flex;
  align-items: center;
  justify-content: flex-end;
  gap: 8px;
  flex-shrink: 0;
  padding: 8px 10px;
  border-bottom: 1px solid var(--border, #313244);
  background: var(--bg-mantle, #181825);
}
.toolbar-btn {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  height: 30px;
  padding: 0 10px;
  border: 1px solid color-mix(in srgb, var(--border, #45475a) 85%, transparent);
  border-radius: 8px;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  font-size: calc(12px + var(--ui-font-size-offset));
  cursor: pointer;
  transition: background-color 0.15s ease, color 0.15s ease, border-color 0.15s ease;
}
.toolbar-btn:hover {
  color: var(--text, #cdd6f4);
  border-color: rgba(137, 180, 250, 0.45);
  background: rgba(137, 180, 250, 0.08);
}
.primary-action-btn,
.secondary-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 90px;
  height: 34px;
  padding: 0 12px;
  border-radius: 10px;
  border: 1px solid color-mix(in srgb, var(--border, #45475a) 85%, transparent);
  cursor: pointer;
  font-size: calc(12px + var(--ui-font-size-offset));
}
.primary-action-btn {
  background: var(--blue, #89b4fa);
  color: var(--button-text-color, #111827);
  border-color: transparent;
}
.secondary-action-btn {
  background: transparent;
  color: var(--text-sub, #a6adc8);
}
.primary-action-btn:hover,
.secondary-action-btn:hover {
  filter: brightness(1.05);
}
.docker-placeholder {
  display: flex;
  flex: 1;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 10px;
  padding: 16px;
  text-align: center;
  color: var(--text-dim, #6c7086);
}
.docker-placeholder i { font-size: calc(32px + var(--ui-font-size-offset)); }
.docker-placeholder p { margin: 0; font-weight: 600; color: var(--text, #cdd6f4); }
.docker-placeholder small { max-width: 80%; font-size: calc(12px + var(--ui-font-size-offset)); line-height: 1.6; }
.docker-placeholder .primary-action-btn { margin-top: 8px; }
.docker-placeholder.error i { color: var(--red, #f38ba8); }
.docker-placeholder.compact { min-height: 100%; }
.docker-content-area { flex: 1; overflow: auto; }
.docker-table { width: 100%; border-collapse: collapse; font-size: calc(13px + var(--ui-font-size-offset)); }
.docker-table thead tr { background: var(--bg-mantle, #181825); }
.docker-table th {
  padding: 9px 12px;
  border-bottom: 1px solid var(--border, #313244);
  text-align: left;
  font-size: calc(11px + var(--ui-font-size-offset));
  font-weight: 600;
  color: var(--text-sub, #a6adc8);
  text-transform: uppercase;
  letter-spacing: .04em;
}
.w-expand { width: 36px; }
.responsive-tr { transition: background-color 0.15s ease; }
.responsive-tr:hover { background: rgba(137, 180, 250, 0.06); }
.responsive-td, .responsive-td-expand { padding: 10px 12px; border-bottom: 1px solid var(--border, #313244); vertical-align: middle; }
.responsive-td-card-expand { display: none; }
.responsive-expansion-row { display: table-row; }
.font-strong { font-weight: 600; }
.break-all { word-break: break-all; }
.icon-btn {
  border: 0;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  transition: color 0.15s ease, background-color 0.15s ease;
}
.icon-btn:hover { color: var(--text, #cdd6f4); background: rgba(255,255,255,.05); }
.icon-btn:disabled { color: var(--text-disabled, #6c7086); cursor: not-allowed; background: transparent; }
.icon-btn.danger:hover { color: var(--red, #f38ba8); }
.action-group { display: flex; justify-content: flex-start; gap: 6px; flex-wrap: wrap; }
.status-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 8px;
  border-radius: 999px;
  font-size: calc(11px + var(--ui-font-size-offset));
  font-weight: 600;
  color: #fff;
  white-space: nowrap;
}
.state-running { background: #22c55e; }
.state-exited { background: #ef4444; }
.state-paused { background: #eab308; color: #1f2937; }
.state-restarting { background: #3b82f6; }
.state-created, .state-removing, .state-dead, .state-unknown { background: #6b7280; }
.expansion-cell { padding: 0; border-bottom: 1px solid var(--border, #313244); }
.stats-box {
  background: color-mix(in srgb, var(--bg-surface0, #313244) 80%, transparent);
  padding: 14px 16px;
}
.stats-grid {
  display: grid;
  grid-template-columns: max-content auto;
  gap: 8px 14px;
  margin: 0;
  font-size: calc(12px + var(--ui-font-size-offset));
}
.stats-grid dt { font-weight: 600; color: var(--text-sub, #a6adc8); }
.stats-grid dd { margin: 0; font-family: 'Cascadia Mono', 'Consolas', monospace; }
.stats-empty { text-align: center; font-size: calc(12px + var(--ui-font-size-offset)); color: var(--text-dim, #6c7086); font-style: italic; }
.card-expand-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  width: 100%;
  height: 40px;
  border: 0;
  background: transparent;
  color: var(--text-sub, #a6adc8);
  cursor: pointer;
}
.card-expand-btn:hover { color: var(--text, #cdd6f4); background: rgba(255,255,255,.04); }
.card-expand-btn.collapse { border-top: 1px solid var(--border, #313244); }
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9200;
  background: rgba(0, 0, 0, 0.45);
  display: flex;
  align-items: center;
  justify-content: center;
}
.sudo-dialog {
  width: min(420px, calc(100vw - 32px));
  border: 1px solid var(--border, #313244);
  border-radius: 14px;
  background: var(--bg-surface0, #313244);
  box-shadow: 0 20px 40px rgba(0, 0, 0, 0.45);
  padding: 18px;
}
.sudo-dialog-title {
  display: flex;
  align-items: center;
  gap: 10px;
  color: var(--text, #cdd6f4);
  font-size: calc(15px + var(--ui-font-size-offset));
  font-weight: 600;
  margin-bottom: 12px;
}
.sudo-dialog-input {
  width: 100%;
  height: 40px;
  border: 1px solid color-mix(in srgb, var(--border, #45475a) 85%, transparent);
  border-radius: 10px;
  background: var(--bg-base, #1e1e2e);
  color: var(--text, #cdd6f4);
  padding: 0 12px;
  font-size: calc(13px + var(--ui-font-size-offset));
  outline: none;
}
.sudo-dialog-input:focus {
  border-color: var(--blue, #89b4fa);
  box-shadow: 0 0 0 2px rgba(137, 180, 250, 0.2);
}
.sudo-dialog-error {
  margin-top: 10px;
  color: var(--red, #f38ba8);
  font-size: calc(12px + var(--ui-font-size-offset));
}
.sudo-dialog-hint {
  margin-top: 10px;
  color: var(--text-dim, #6c7086);
  font-size: calc(12px + var(--ui-font-size-offset));
  line-height: 1.6;
}
.sudo-dialog-actions {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
  margin-top: 16px;
}

@container docker-manager-pane (max-width: 600px) {
  .docker-content-area { padding: 12px; }
  .responsive-thead { display: none; }
  .responsive-tbody { display: block; }
  .responsive-tr {
    display: block;
    margin-bottom: 14px;
    border: 1px solid var(--border, #313244);
    border-radius: 12px;
    padding: 12px;
    background: var(--bg-base, #1e1e2e);
    box-shadow: 0 6px 18px rgba(0, 0, 0, 0.18);
  }
  .responsive-td {
    display: block;
    position: relative;
    padding: 8px 0 8px 46%;
    border-bottom: 1px dashed color-mix(in srgb, var(--border, #313244) 70%, transparent);
    text-align: right;
  }
  .responsive-td:last-of-type { border-bottom: none; }
  .responsive-td::before {
    content: attr(data-label);
    position: absolute;
    left: 0;
    width: calc(46% - 10px);
    text-align: left;
    font-weight: 600;
    color: var(--text-sub, #a6adc8);
  }
  .responsive-td-expand, .responsive-expansion-row { display: none; }
  .responsive-td-card-expand { display: block; width: 100%; margin-top: 10px; }
  .action-group { justify-content: flex-end; padding-top: 8px; }
}
</style>

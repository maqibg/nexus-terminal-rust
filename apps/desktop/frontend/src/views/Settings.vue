<template>
  <div class="settings-page">
    <div class="settings-container">
      <!-- Horizontal Tabs -->
      <div class="tabs-bar">
        <button
          v-for="tab in navTabs"
          :key="tab.key"
          class="tab-btn"
          :class="{ active: activeTab === tab.key }"
          @click="activeTab = tab.key"
        >{{ tab.label }}</button>
      </div>

      <!-- Tab Content -->
      <div class="tab-content">
        <!-- 通用设置 -->
        <section v-if="activeTab === 'general'" class="section-card">
          <h3 class="section-title">通用设置</h3>
          <div v-if="loadingSettings" class="status">加载中...</div>
          <div v-else class="setting-list">
            <div v-for="s in settings" :key="s.key" class="setting-row">
              <label>{{ s.key }}</label>
              <input :value="s.value" @change="updateSetting(s.key, ($event.target as HTMLInputElement).value)" />
            </div>
            <div v-if="!settings.length" class="status">暂无设置项</div>
          </div>
        </section>

        <!-- 外观 -->
        <section v-if="activeTab === 'appearance'" class="section-card">
          <div class="section-header">
            <h3 class="section-title">外观设置</h3>
            <button class="btn-action" @click="showStyleCustomizer = true">样式定制</button>
          </div>
          <div v-if="loadingAppearance" class="status">加载中...</div>
          <div v-else class="setting-list">
            <div v-for="s in appearance" :key="s.key" class="setting-row">
              <label>{{ s.key }}</label>
              <input :value="s.value" @change="updateAppearance(s.key, ($event.target as HTMLInputElement).value)" />
            </div>
            <div v-if="!appearance.length" class="status">暂无外观设置</div>
          </div>
        </section>

        <!-- 终端主题 -->
        <section v-if="activeTab === 'themes'" class="section-card">
          <h3 class="section-title">终端主题</h3>
          <div v-if="loadingThemes" class="status">加载中...</div>
          <div v-else class="theme-list">
            <div v-for="t in themes" :key="t.id" class="theme-card">
              <div class="theme-preview" :style="{ background: t.background ?? '#1e1e1e', color: t.foreground ?? '#d4d4d4' }">
                <span>{{ t.name }}</span>
                <span class="theme-type">{{ t.theme_type }}</span>
              </div>
              <div class="theme-colors">
                <span v-for="c in themeColors(t)" :key="c" class="color-dot" :style="{ background: c }" :title="c"></span>
              </div>
            </div>
            <div v-if="!themes.length" class="status">暂无主题</div>
          </div>
        </section>

        <!-- SSH 密钥 -->
        <section v-if="activeTab === 'sshkeys'" class="section-card">
          <div class="section-header">
            <h3 class="section-title">SSH 密钥</h3>
            <button class="btn-action" @click="showSshKeyModal = true">管理密钥</button>
          </div>
          <div v-if="loadingKeys" class="status">加载中...</div>
          <div v-else class="item-list">
            <div v-for="k in sshKeys" :key="k.id" class="item-card">
              <span class="item-name">{{ k.name }}</span>
              <button class="btn-del" @click="deleteSshKey(k.id, k.name)">删除</button>
            </div>
            <div v-if="!sshKeys.length" class="status">暂无 SSH 密钥</div>
          </div>
        </section>

        <!-- 代理 -->
        <section v-if="activeTab === 'proxies'" class="section-card">
          <div class="section-header">
            <h3 class="section-title">代理</h3>
            <button class="btn-action" @click="editingProxy = undefined; showProxyForm = true">新建代理</button>
          </div>
          <div v-if="loadingProxies" class="status">加载中...</div>
          <div v-else class="item-list">
            <div v-for="p in proxies" :key="p.id" class="item-card">
              <div class="item-info">
                <span class="item-name">{{ p.name }}</span>
                <span class="item-sub">{{ p.proxy_type }} — {{ p.host }}:{{ p.port }}</span>
              </div>
              <div class="item-actions">
                <button class="btn-edit" @click="editingProxy = p; showProxyForm = true">编辑</button>
                <button class="btn-del" @click="deleteProxy(p.id, p.name)">删除</button>
              </div>
            </div>
            <div v-if="!proxies.length" class="status">暂无代理</div>
          </div>
        </section>

        <!-- 标签 -->
        <section v-if="activeTab === 'tags'" class="section-card">
          <div class="section-header">
            <h3 class="section-title">标签</h3>
            <button class="btn-action" @click="showTagConnModal = true">管理标签-连接</button>
          </div>
          <div class="tag-add">
            <input v-model="newTagName" placeholder="新标签名称" @keydown.enter="createTag" />
            <button class="btn-save" @click="createTag" :disabled="!newTagName.trim()">创建</button>
          </div>
          <div v-if="loadingTags" class="status">加载中...</div>
          <div v-else class="item-list">
            <div v-for="t in tags" :key="t.id" class="item-card">
              <span class="item-name">{{ t.name }}</span>
              <button class="btn-del" @click="deleteTag(t.id, t.name)">删除</button>
            </div>
            <div v-if="!tags.length" class="status">暂无标签</div>
          </div>
        </section>

        <!-- 通知渠道 -->
        <section v-if="activeTab === 'notifications'" class="section-card">
          <div class="section-header">
            <h3 class="section-title">通知渠道</h3>
            <button class="btn-action" @click="editingChannel = undefined; showChannelForm = true">新建渠道</button>
          </div>
          <div v-if="loadingChannels" class="status">加载中...</div>
          <div v-else class="item-list">
            <div v-for="ch in channels" :key="ch.id" class="item-card">
              <div class="item-info">
                <span class="item-name">{{ ch.name }}</span>
                <span class="item-sub">{{ ch.channel_type }}</span>
              </div>
              <div class="item-actions">
                <span class="channel-status" :class="{ enabled: ch.enabled }">{{ ch.enabled ? '启用' : '禁用' }}</span>
                <button class="btn-edit" @click="editingChannel = ch; showChannelForm = true">编辑</button>
                <button class="btn-del" @click="deleteChannel(ch.id, ch.name)">删除</button>
              </div>
            </div>
            <div v-if="!channels.length" class="status">暂无通知渠道</div>
          </div>
        </section>

        <!-- 安全 -->
        <section v-if="activeTab === 'security'" class="section-card">
          <h3 class="section-title">修改密码</h3>
          <div class="form-group">
            <label>当前密码 <input v-model="pwForm.current" type="password" /></label>
            <label>新密码 <input v-model="pwForm.newPw" type="password" /></label>
            <label>确认新密码 <input v-model="pwForm.confirm" type="password" /></label>
            <div v-if="pwError" class="error">{{ pwError }}</div>
            <div v-if="pwSuccess" class="success">密码已更新</div>
            <button class="btn-save" @click="changePassword" :disabled="pwBusy">修改密码</button>
          </div>
          <hr class="divider" />
          <PasskeyManagement />
        </section>

        <!-- 数据管理 -->
        <section v-if="activeTab === 'data'" class="section-card">
          <DataManagementSection />
        </section>

        <!-- 快捷键 -->
        <section v-if="activeTab === 'shortcuts'" class="section-card">
          <div class="section-header">
            <h3 class="section-title">配置焦点切换器</h3>
            <button class="btn-action" @click="focusSwitcherStore.toggleConfigurator(true)">打开配置器</button>
          </div>
          <p class="status">按下 Alt 键可按序切换焦点；按 Alt+字母/数字可快速定位到指定输入源。</p>
          <p class="status">配置入口与原版一致，可在终端底部命令栏点击键盘图标快速打开。</p>
        </section>

        <!-- 关于 -->
        <section v-if="activeTab === 'about'" class="section-card">
          <AboutSection />
        </section>
      </div>
    </div>

    <!-- Modals -->
    <StyleCustomizer :visible="showStyleCustomizer" @close="showStyleCustomizer = false" />
    <SshKeyManagementModal :visible="showSshKeyModal" @close="showSshKeyModal = false; loadSshKeys()" />
    <AddProxyForm :visible="showProxyForm" :proxy="editingProxy" @saved="showProxyForm = false; loadProxies()" @cancel="showProxyForm = false" />
    <NotificationSettingForm :visible="showChannelForm" :channel="editingChannel" @saved="showChannelForm = false; loadChannels()" @cancel="showChannelForm = false" />
    <ManageTagConnectionsModal :visible="showTagConnModal" @close="showTagConnModal = false" />
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue';
import { settingsApi, authApi, connectionsApi } from '@/lib/api';
import type { Setting, TerminalTheme, NotificationChannel, SshKey, Proxy, Tag } from '@/lib/api';
import StyleCustomizer from '@/components/StyleCustomizer.vue';
import SshKeyManagementModal from '@/components/SshKeyManagementModal.vue';
import AddProxyForm from '@/components/AddProxyForm.vue';
import NotificationSettingForm from '@/components/NotificationSettingForm.vue';
import ManageTagConnectionsModal from '@/components/ManageTagConnectionsModal.vue';
import DataManagementSection from '@/components/DataManagementSection.vue';
import PasskeyManagement from '@/components/PasskeyManagement.vue';
import AboutSection from '@/components/AboutSection.vue';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';

const navTabs = [
  { key: 'general', label: '通用' },
  { key: 'appearance', label: '外观' },
  { key: 'themes', label: '终端主题' },
  { key: 'sshkeys', label: 'SSH 密钥' },
  { key: 'proxies', label: '代理' },
  { key: 'tags', label: '标签' },
  { key: 'notifications', label: '通知' },
  { key: 'security', label: '安全' },
  { key: 'data', label: '数据管理' },
  { key: 'shortcuts', label: '快捷键' },
  { key: 'about', label: '关于' },
] as const;

const activeTab = ref<string>('general');
const focusSwitcherStore = useFocusSwitcherStore();

// Modal states
const showStyleCustomizer = ref(false);
const showSshKeyModal = ref(false);
const showProxyForm = ref(false);
const showChannelForm = ref(false);
const showTagConnModal = ref(false);
const editingProxy = ref<Proxy | undefined>();
const editingChannel = ref<NotificationChannel | undefined>();

// General settings
const settings = ref<Setting[]>([]);
const loadingSettings = ref(false);
async function loadSettings() {
  loadingSettings.value = true;
  try { settings.value = await settingsApi.getAll(); } catch { /* ignore */ }
  finally { loadingSettings.value = false; }
}
async function updateSetting(key: string, value: string) {
  try { await settingsApi.set(key, value); const s = settings.value.find(i => i.key === key); if (s) s.value = value; } catch { /* ignore */ }
}

// Appearance
const appearance = ref<Setting[]>([]);
const loadingAppearance = ref(false);
async function loadAppearance() {
  loadingAppearance.value = true;
  try { appearance.value = await settingsApi.appearanceGetAll(); } catch { /* ignore */ }
  finally { loadingAppearance.value = false; }
}
async function updateAppearance(key: string, value: string) {
  try { await settingsApi.appearanceSet(key, value); const s = appearance.value.find(i => i.key === key); if (s) s.value = value; } catch { /* ignore */ }
}

// Themes
const themes = ref<TerminalTheme[]>([]);
const loadingThemes = ref(false);
async function loadThemes() {
  loadingThemes.value = true;
  try { themes.value = await settingsApi.themeList(); } catch { /* ignore */ }
  finally { loadingThemes.value = false; }
}
function themeColors(t: TerminalTheme): string[] {
  return [t.black, t.red, t.green, t.yellow, t.blue, t.magenta, t.cyan, t.white]
    .filter((value): value is string => typeof value === 'string' && value.length > 0);
}

// Security
const pwForm = reactive({ current: '', newPw: '', confirm: '' });
const pwError = ref('');
const pwSuccess = ref(false);
const pwBusy = ref(false);

// Notifications
const channels = ref<NotificationChannel[]>([]);
const loadingChannels = ref(false);
async function loadChannels() {
  loadingChannels.value = true;
  try { channels.value = await settingsApi.notificationChannelList(); } catch { /* ignore */ }
  finally { loadingChannels.value = false; }
}
async function deleteChannel(id: number, name: string) {
  if (!confirm(`确定删除通知渠道 "${name}"？`)) return;
  try { await settingsApi.notificationChannelDelete(id); loadChannels(); } catch { /* ignore */ }
}

// SSH Keys
const sshKeys = ref<SshKey[]>([]);
const loadingKeys = ref(false);
async function loadSshKeys() {
  loadingKeys.value = true;
  try { sshKeys.value = await connectionsApi.sshKeyList(); } catch { /* ignore */ }
  finally { loadingKeys.value = false; }
}
async function deleteSshKey(id: number, name: string) {
  if (!confirm(`确定删除 SSH 密钥 "${name}"？`)) return;
  try { await connectionsApi.sshKeyDelete(id); loadSshKeys(); } catch { /* ignore */ }
}

// Proxies
const proxies = ref<Proxy[]>([]);
const loadingProxies = ref(false);
async function loadProxies() {
  loadingProxies.value = true;
  try { proxies.value = await connectionsApi.proxyList(); } catch { /* ignore */ }
  finally { loadingProxies.value = false; }
}
async function deleteProxy(id: number, name: string) {
  if (!confirm(`确定删除代理 "${name}"？`)) return;
  try { await connectionsApi.proxyDelete(id); loadProxies(); } catch { /* ignore */ }
}

// Tags
const tags = ref<Tag[]>([]);
const loadingTags = ref(false);
const newTagName = ref('');
async function loadTags() {
  loadingTags.value = true;
  try { tags.value = await connectionsApi.tagList(); } catch { /* ignore */ }
  finally { loadingTags.value = false; }
}
async function createTag() {
  if (!newTagName.value.trim()) return;
  try { await connectionsApi.tagCreate(newTagName.value.trim()); newTagName.value = ''; loadTags(); } catch { /* ignore */ }
}
async function deleteTag(id: number, name: string) {
  if (!confirm(`确定删除标签 "${name}"？`)) return;
  try { await connectionsApi.tagDelete(id); loadTags(); } catch { /* ignore */ }
}

async function changePassword() {
  pwError.value = ''; pwSuccess.value = false;
  if (!pwForm.current || !pwForm.newPw) { pwError.value = '请填写所有字段'; return; }
  if (pwForm.newPw !== pwForm.confirm) { pwError.value = '新密码不一致'; return; }
  pwBusy.value = true;
  try {
    await authApi.changePassword(pwForm.current, pwForm.newPw);
    pwSuccess.value = true;
    pwForm.current = ''; pwForm.newPw = ''; pwForm.confirm = '';
  } catch (e: any) { pwError.value = e.message; }
  finally { pwBusy.value = false; }
}

onMounted(() => {
  loadSettings(); loadAppearance(); loadThemes();
  loadChannels(); loadSshKeys(); loadProxies(); loadTags();
});
</script>

<style scoped>
.settings-page { padding: 20px 24px; height: 100%; overflow-y: auto; color: var(--text); }
.settings-container { max-width: 960px; margin: 0 auto; }

.tabs-bar {
  display: flex; flex-wrap: wrap; gap: 4px;
  padding: 8px 0; margin-bottom: 16px;
}
.tab-btn {
  padding: 8px 16px; font-size: 13px; font-weight: 500;
  border-radius: 6px; border: none; cursor: pointer;
  background: transparent; color: var(--text-sub);
  transition: all 0.15s;
}
.tab-btn:hover { background: var(--bg-surface1); color: var(--text); }
.tab-btn.active { background: var(--blue); color: #fff; }

.tab-content { }

.section-card {
  background: var(--bg-surface0); border: 1px solid var(--border);
  border-radius: 8px; padding: 20px 24px; margin-bottom: 16px;
}
.section-title { margin: 0 0 12px; font-size: 15px; font-weight: 600; color: var(--text); }
.section-header { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.section-header .section-title { margin: 0; }

.btn-action {
  padding: 5px 14px; border-radius: 4px; border: 1px solid var(--blue);
  background: transparent; color: var(--blue); cursor: pointer; font-size: 12px;
}
.btn-action:hover { background: rgba(137,180,250,0.1); }

.setting-list { display: flex; flex-direction: column; gap: 8px; }
.setting-row { display: flex; align-items: center; gap: 12px; }
.setting-row label { min-width: 160px; font-size: 13px; color: var(--text-sub); }
.setting-row input {
  flex: 1; max-width: 320px; padding: 6px 10px; border-radius: 4px;
  border: 1px solid var(--border); background: var(--bg-base); color: var(--text);
  font-size: 13px; outline: none;
}
.setting-row input:focus { border-color: var(--blue); }

.status { color: var(--text-dim); font-size: 13px; padding: 8px 0; }

.theme-list { display: flex; flex-wrap: wrap; gap: 12px; }
.theme-card { border: 1px solid var(--border); border-radius: 8px; overflow: hidden; width: 200px; }
.theme-preview { padding: 12px; display: flex; justify-content: space-between; align-items: center; font-size: 13px; }
.theme-type { font-size: 11px; opacity: 0.6; }
.theme-colors { display: flex; gap: 4px; padding: 8px; background: var(--bg-base); }
.color-dot { width: 16px; height: 16px; border-radius: 50%; border: 1px solid var(--border); }

.form-group { display: flex; flex-direction: column; gap: 8px; max-width: 320px; }
.form-group label { display: flex; flex-direction: column; gap: 4px; font-size: 13px; color: var(--text-sub); }
.form-group input {
  padding: 6px 10px; border-radius: 4px; border: 1px solid var(--border);
  background: var(--bg-base); color: var(--text); font-size: 13px; outline: none;
}
.form-group input:focus { border-color: var(--blue); }
.btn-save {
  padding: 6px 16px; border-radius: 6px; border: none; align-self: flex-start;
  background: var(--blue); color: #fff; cursor: pointer; font-weight: 600; font-size: 13px;
}
.btn-save:hover { opacity: 0.9; }
.btn-save:disabled { opacity: 0.5; cursor: not-allowed; }
.error { color: var(--red); font-size: 13px; }
.success { color: var(--green); font-size: 13px; }
.divider { border: none; border-top: 1px solid var(--border); margin: 16px 0; }

.item-list { display: flex; flex-direction: column; gap: 6px; }
.item-card {
  display: flex; align-items: center; justify-content: space-between;
  padding: 10px 14px; border: 1px solid var(--border); border-radius: 6px;
  background: var(--bg-surface1);
}
.item-info { display: flex; flex-direction: column; gap: 2px; }
.item-name { font-size: 13px; color: var(--text); }
.item-sub { font-size: 11px; color: var(--text-dim); }
.item-actions { display: flex; align-items: center; gap: 6px; }
.btn-edit {
  padding: 3px 10px; border-radius: 4px; border: 1px solid var(--blue);
  background: transparent; color: var(--blue); cursor: pointer; font-size: 12px;
}
.btn-edit:hover { background: rgba(137,180,250,0.15); }
.btn-del {
  padding: 3px 10px; border-radius: 4px; border: 1px solid var(--red);
  background: transparent; color: var(--red); cursor: pointer; font-size: 12px;
}
.btn-del:hover { background: rgba(243,139,168,0.15); }
.channel-status { font-size: 12px; color: var(--red); }
.channel-status.enabled { color: var(--green); }
.tag-add { display: flex; gap: 8px; margin-bottom: 12px; }
.tag-add input {
  flex: 1; max-width: 240px; padding: 6px 10px; border-radius: 4px;
  border: 1px solid var(--border); background: var(--bg-base); color: var(--text);
  font-size: 13px; outline: none;
}
.tag-add input:focus { border-color: var(--blue); }
</style>

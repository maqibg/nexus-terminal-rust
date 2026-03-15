<template>
  <UINotificationDisplay />
  <GlobalAlertDialog />
  <GlobalConfirmDialog />
  <SshKeyConfirmModal />

  <div id="app-container">
    <header v-if="showHeader" class="app-header" data-tauri-drag-region>
      <nav class="app-nav" data-tauri-drag-region>
        <div class="nav-left no-drag">
          <img :src="logoPng" alt="Nexus Terminal" class="brand-logo" title="Nexus Terminal" />
          <router-link to="/connections" class="nav-link" active-class="nav-link-active">{{ uiText.navConnections }}</router-link>
          <router-link to="/workspace" class="nav-link" active-class="nav-link-active">{{ uiText.navTerminal }}</router-link>
          <router-link to="/databases" class="nav-link" active-class="nav-link-active">{{ uiText.navDatabases }}</router-link>
          <router-link to="/tools" class="nav-link" active-class="nav-link-active">{{ uiText.navTools }}</router-link>
          <router-link to="/proxies" class="nav-link nav-link-desktop" active-class="nav-link-active">{{ uiText.navProxy }}</router-link>
          <router-link to="/statistics" class="nav-link" active-class="nav-link-active">{{ uiText.navStatistics }}</router-link>
          <router-link to="/settings" class="nav-link" active-class="nav-link-active">{{ uiText.navSettings }}</router-link>
        </div>

        <div class="nav-drag-region" data-tauri-drag-region></div>

        <div class="nav-right no-drag">
          <a
            class="nav-icon-btn"
            href="https://github.com/maqibg/nexus-terminal-rust"
            target="_blank"
            rel="noopener noreferrer"
            title="maqibg/nexus-terminal-rust"
          >
            <i class="fab fa-github"></i>
          </a>
          <button
            class="nav-icon-btn"
            :class="{ active: showGlobalAiPanel }"
            :title="uiText.aiAssistant"
            @click="toggleGlobalAiPanel"
          >
            <i class="fas fa-robot"></i>
          </button>
          <button class="nav-icon-btn" :title="uiText.customizeAppearance" @click="appearanceStore.toggleStyleCustomizer(true)">
            <i class="fas fa-paint-brush"></i>
          </button>
          <button v-if="isAuthenticated" @click="handleLogout" class="nav-link nav-link-ghost">{{ uiText.logout }}</button>

          <div class="window-controls">
            <button class="window-btn minimize" @click="minimizeWindow" :title="uiText.minimize" :aria-label="uiText.minimize">
              <svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <path fill-rule="evenodd" d="M2 8a.5.5 0 0 1 .5-.5h11a.5.5 0 0 1 0 1h-11A.5.5 0 0 1 2 8Z" />
              </svg>
            </button>
            <button class="window-btn maximize" @click="toggleMaximize" :title="uiText.maximize" :aria-label="uiText.maximize">
              <svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <path d="M14 1a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1h12zM2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2H2z" />
              </svg>
            </button>
            <button class="window-btn close" @click="closeWindow" :title="uiText.close" :aria-label="uiText.close">
              <svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <path d="M2.146 2.854a.5.5 0 1 1 .708-.708L8 7.293l5.146-5.147a.5.5 0 0 1 .708.708L8.707 8l5.147 5.146a.5.5 0 0 1-.708.708L8 8.707l-5.146 5.147a.5.5 0 0 1-.708-.708L7.293 8 2.146 2.854Z" />
              </svg>
            </button>
          </div>
        </div>
      </nav>
    </header>

    <div
      v-if="startupState !== 'ready'"
      class="startup-banner"
      :class="startupState"
    >
      <span v-if="startupState === 'starting'">{{ uiText.backendStarting }}</span>
      <span v-else>{{ uiText.backendFailed }}{{ startupError || uiText.unknownError }}</span>
    </div>

    <main class="app-main">
      <router-view v-slot="{ Component, route }">
        <KeepAlive>
          <component :is="Component" v-if="route.path === '/workspace'" :key="route.path" />
        </KeepAlive>
        <component :is="Component" v-if="route.path !== '/workspace'" :key="route.path" />
      </router-view>
    </main>

    <transition name="global-ai-slide">
      <aside v-if="showGlobalAiPanel && isAuthenticated" class="global-ai-panel no-drag" :style="globalAiPanelStyle">
        <TerminalAIChatPanel ref="globalAiPanelRef" storage-id="global" :closable="true" @close="showGlobalAiPanel = false" />
      </aside>
    </transition>

    <StyleCustomizer v-if="isStyleCustomizerVisible" :visible="isStyleCustomizerVisible" @close="appearanceStore.toggleStyleCustomizer(false)" />

    <FocusSwitcherConfigurator
      :visible="isFocusSwitcherVisible"
      @close="focusSwitcherStore.toggleConfigurator(false)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, onUnmounted, ref, watch } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAuthStore } from '@/stores/auth';
import { useLayoutStore } from '@/stores/layout';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { useAppearanceStore } from '@/stores/appearance';
import { useSettingsStore } from '@/stores/settings';
import { statusApi } from '@/lib/api';
import UINotificationDisplay from '@/components/UINotificationDisplay.vue';
import GlobalAlertDialog from '@/components/GlobalAlertDialog.vue';
import GlobalConfirmDialog from '@/components/GlobalConfirmDialog.vue';
import SshKeyConfirmModal from '@/components/SshKeyConfirmModal.vue';
import FocusSwitcherConfigurator from '@/components/FocusSwitcherConfigurator.vue';
import StyleCustomizer from '@/components/StyleCustomizer.vue';
import TerminalAIChatPanel from '@/components/AI/TerminalAIChatPanel.vue';
import logoPng from '@/assets/logo.png';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const layoutStore = useLayoutStore();
const focusSwitcherStore = useFocusSwitcherStore();
const appearanceStore = useAppearanceStore();
const settingsStore = useSettingsStore();
const { isAuthenticated } = storeToRefs(authStore);
const { headerVisible } = storeToRefs(layoutStore);
const { isConfiguratorVisible: isFocusSwitcherVisible } = storeToRefs(focusSwitcherStore);
const { isStyleCustomizerVisible } = storeToRefs(appearanceStore);
const appWindow = getCurrentWindow();
const uiText = computed(() => ({
  navConnections: '连接管理',
  navTerminal: '终端',
  navDatabases: '数据库',
  navTools: '工具箱',
  navProxy: '代理管理',
  navStatistics: '统计分析',
  navSettings: '设置',
  aiAssistant: 'AI 助手',
  customizeAppearance: '外观自定义',
  logout: '登出',
  minimize: '最小化',
  maximize: '最大化/还原',
  close: '关闭',
  backendStarting: '后端启动中...',
  backendFailed: '后端启动失败：',
  unknownError: '未知错误',
}));

const noHeaderPaths = ['/login', '/setup'];
const showHeader = computed(() => {
  if (noHeaderPaths.includes(route.path)) {
    return false;
  }

  if (route.path === '/workspace') {
    return headerVisible.value;
  }

  return true;
});
const isWorkspaceRoute = computed(() => route.path === '/workspace');

const startupState = ref<'starting' | 'ready' | 'error'>('starting');
const startupError = ref('');

const isAltPressed = ref(false);
const altShortcutKey = ref<string | null>(null);
const lastFocusedIdBySwitcher = ref<string | null>(null);
const showGlobalAiPanel = ref(false);

interface GlobalAiPanelExpose {
  setInput: (value: string) => void;
  sendMessage: (override?: string) => Promise<void>;
  performAction: (text: string) => Promise<void>;
}
interface GlobalAiOpenDetail {
  prompt?: string;
  autoSend?: boolean;
}
const globalAiPanelRef = ref<GlobalAiPanelExpose | null>(null);

const globalAiPanelStyle = computed<Record<string, string>>(() => {
  const panelTop = showHeader.value ? 55 : 0;
  return {
    top: `${panelTop}px`,
    height: `calc(100vh - ${panelTop}px)`,
  };
});

async function checkBackendStartup() {
  startupState.value = 'starting';
  startupError.value = '';
  try {
    const health = await statusApi.getBackendHealth();
    if (health.status === 'ok') {
      startupState.value = 'ready';
    } else {
      startupState.value = 'error';
      startupError.value = `status=${health.status}`;
    }
  } catch (e: unknown) {
    startupState.value = 'error';
    startupError.value = e instanceof Error ? e.message : String(e);
  }
}

function resolveFocusIdFromActiveElement(): string | null {
  let current = document.activeElement as HTMLElement | null;
  while (current) {
    if (current.hasAttribute('data-focus-id')) {
      return current.getAttribute('data-focus-id');
    }
    current = current.parentElement;
  }
  return null;
}

async function handleAltKeyDown(event: KeyboardEvent): Promise<void> {
  if (!isWorkspaceRoute.value || isFocusSwitcherVisible.value) {
    return;
  }

  if (event.key === 'Alt' && !event.repeat) {
    isAltPressed.value = true;
    altShortcutKey.value = null;
    return;
  }

  if (!isAltPressed.value) {
    return;
  }

  if (['Control', 'Shift', 'Meta'].includes(event.key)) {
    isAltPressed.value = false;
    altShortcutKey.value = null;
    return;
  }

  if (['Alt'].includes(event.key)) {
    return;
  }

  let key = event.key;
  if (key.length === 1) {
    key = key.toUpperCase();
  }

  if (!/^[A-Z0-9]$/.test(key)) {
    isAltPressed.value = false;
    altShortcutKey.value = null;
    return;
  }

  altShortcutKey.value = key;
  const targetId = focusSwitcherStore.getFocusTargetIdByShortcut(`Alt+${key}`);
  if (!targetId) {
    return;
  }

  event.preventDefault();
  const focused = await focusSwitcherStore.focusTarget(targetId);
  if (focused) {
    lastFocusedIdBySwitcher.value = targetId;
  }
}

async function handleAltKeyUp(event: KeyboardEvent): Promise<void> {
  if (!isWorkspaceRoute.value || isFocusSwitcherVisible.value || event.key !== 'Alt') {
    return;
  }

  const altWasPressed = isAltPressed.value;
  const shortcutKey = altShortcutKey.value;

  isAltPressed.value = false;
  altShortcutKey.value = null;

  if (!altWasPressed || shortcutKey !== null) {
    return;
  }

  event.preventDefault();

  let currentFocusId = lastFocusedIdBySwitcher.value ?? resolveFocusIdFromActiveElement();
  const order = focusSwitcherStore.sequenceOrder;
  if (order.length === 0) {
    return;
  }

  for (let i = 0; i < order.length; i += 1) {
    const nextFocusId = focusSwitcherStore.getNextFocusTargetId(currentFocusId);
    if (!nextFocusId) {
      break;
    }

    const focused = await focusSwitcherStore.focusTarget(nextFocusId);
    if (focused) {
      lastFocusedIdBySwitcher.value = nextFocusId;
      return;
    }

    currentFocusId = nextFocusId;
  }

  lastFocusedIdBySwitcher.value = null;
}

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}

async function openGlobalAiPanel(detail?: GlobalAiOpenDetail) {
  if (!isAuthenticated.value) {
    return;
  }
  showGlobalAiPanel.value = true;

  const prompt = detail?.prompt?.trim();
  if (!prompt) {
    return;
  }

  await nextTick();
  const panel = globalAiPanelRef.value;
  if (!panel) {
    return;
  }

  if (detail?.autoSend) {
    await panel.performAction(prompt);
    return;
  }

  panel.setInput(prompt);
}

function toggleGlobalAiPanel() {
  if (!isAuthenticated.value) {
    return;
  }
  showGlobalAiPanel.value = !showGlobalAiPanel.value;
}

function handleOpenGlobalAiPanel(event: Event): void {
  const detail = (event as CustomEvent<GlobalAiOpenDetail>).detail;
  void openGlobalAiPanel(detail);
}

async function handleLogout() {
  await authStore.logout();
  router.push('/login');
}

function preventBrowserContextMenu(event: MouseEvent): void {
  event.preventDefault();
}

const loadAppearanceData = async () => {
  await appearanceStore.loadAll().catch(() => undefined);
};

watch(isAuthenticated, (authenticated) => {
  if (!authenticated) {
    showGlobalAiPanel.value = false;
    return;
  }
  void loadAppearanceData();
});

onMounted(() => {
  if (isAuthenticated.value) {
    void loadAppearanceData();
  }
  void settingsStore.loadAll().catch(() => undefined);
  void layoutStore.loadLayout().catch(() => undefined);
  void checkBackendStartup();
  void focusSwitcherStore.loadConfigurationFromBackend();
  window.addEventListener('keydown', handleAltKeyDown);
  window.addEventListener('keyup', handleAltKeyUp);
  window.addEventListener('contextmenu', preventBrowserContextMenu);
  window.addEventListener('nexus:global-ai-assistant:open', handleOpenGlobalAiPanel as EventListener);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleAltKeyDown);
  window.removeEventListener('keyup', handleAltKeyUp);
  window.removeEventListener('contextmenu', preventBrowserContextMenu);
  window.removeEventListener('nexus:global-ai-assistant:open', handleOpenGlobalAiPanel as EventListener);
});
</script>

<style scoped>
#app-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  background: var(--bg-base);
}

.app-header {
  -webkit-app-region: drag;
  position: sticky;
  top: 0;
  z-index: 10;
  height: 55px;
  background: var(--header-bg-color);
  border-bottom: 1px solid var(--border);
}

.app-nav {
  -webkit-app-region: drag;
  display: flex;
  align-items: center;
  height: 100%;
  padding: 0 8px 0 10px;
}

.nav-left,
.nav-right {
  display: flex;
  align-items: center;
  gap: 2px;
}

.brand-logo {
  height: 40px;
  width: auto;
  display: block;
  margin-right: 4px;
  user-select: none;
}

.nav-drag-region {
  flex: 1;
  height: 100%;
}

.nav-link {
  display: inline-flex;
  align-items: center;
  height: 32px;
  padding: 0 10px;
  border-radius: 6px;
  font-size: calc(13px + var(--ui-font-size-offset));
  font-weight: 500;
  color: var(--text-sub);
  text-decoration: none;
  background: transparent;
  border: none;
  cursor: pointer;
  transition: all 0.15s ease;
  white-space: nowrap;
}

.nav-link:hover {
  color: var(--text);
  background: var(--bg-surface1);
}

.nav-link-active {
  color: var(--blue);
  background: var(--link-active-bg-color);
}

.nav-link-ghost {
  padding: 0 10px;
}

.nav-icon-btn {
  width: 32px;
  height: 32px;
  border: none;
  background: transparent;
  border-radius: 6px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  text-decoration: none;
  color: var(--text-sub);
  transition: all 0.15s;
}

.nav-icon-btn:hover {
  color: var(--text);
  background: var(--bg-surface1);
}

.nav-icon-btn.active {
  color: var(--link-active-color);
  background: var(--link-active-bg-color);
}

.window-controls {
  display: flex;
  align-items: center;
  margin-left: 2px;
  padding-left: 0;
  gap: 2px;
}

.window-btn {
  width: 30px;
  height: 26px;
  border: none;
  border-radius: 6px;
  background: transparent;
  color: var(--text-dim);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.window-btn:hover {
  background: var(--bg-surface1);
  color: var(--text);
}

.window-btn.close:hover {
  color: var(--button-text-color);
  background: var(--color-error);
}

.window-btn svg {
  width: 14px;
  height: 14px;
}

.window-btn.maximize svg {
  width: 12px;
  height: 12px;
}

.window-btn:focus-visible {
  outline: 1px solid var(--link-active-color);
  outline-offset: 1px;
}

.no-drag {
  -webkit-app-region: no-drag;
}

.startup-banner {
  padding: 6px 12px;
  font-size: calc(12px + var(--ui-font-size-offset));
  border-bottom: 1px solid var(--border);
}

.startup-banner.starting {
  background: var(--link-active-bg-color);
  color: var(--link-hover-color);
}

.startup-banner.error {
  background: var(--ui-danger-hover);
  color: var(--color-error);
}

.app-main {
  flex: 1;
  overflow: hidden;
}

.global-ai-panel {
  position: fixed;
  right: 0;
  width: min(430px, 42vw);
  min-width: 360px;
  max-width: 520px;
  border-left: 1px solid var(--border);
  background: var(--bg-surface0);
  z-index: 30;
  box-shadow: -16px 0 32px color-mix(in srgb, var(--bg-base) 82%, transparent);
}

.global-ai-slide-enter-active,
.global-ai-slide-leave-active {
  transition: transform 0.2s ease, opacity 0.2s ease;
}

.global-ai-slide-enter-from,
.global-ai-slide-leave-to {
  transform: translateX(100%);
  opacity: 0;
}

@media (max-width: 900px) {
  .app-nav {
    padding-right: 4px;
  }

  .nav-left {
    gap: 0;
  }

  .nav-link {
    padding: 0 8px;
    font-size: calc(12px + var(--ui-font-size-offset));
  }

  .nav-link-desktop {
    display: none;
  }

  .global-ai-panel {
    width: min(420px, 86vw);
    min-width: 320px;
  }
}
</style>

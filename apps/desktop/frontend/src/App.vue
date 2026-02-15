<template>
  <UINotificationDisplay />
  <GlobalAlertDialog />
  <GlobalConfirmDialog />

  <div id="app-container">
    <header v-if="showHeader" class="app-header" data-tauri-drag-region>
      <nav class="app-nav" data-tauri-drag-region>
        <div class="nav-left no-drag">
          <div class="brand-mark" title="Nexus Terminal">
            <i class="fas fa-angle-left"></i>
          </div>
          <router-link to="/connections" class="nav-link" active-class="nav-link-active">连接管理</router-link>
          <router-link to="/workspace" class="nav-link" active-class="nav-link-active">终端</router-link>
          <router-link to="/proxies" class="nav-link nav-link-desktop" active-class="nav-link-active">代理管理</router-link>
          <router-link to="/settings" class="nav-link" active-class="nav-link-active">设置</router-link>
        </div>

        <div class="nav-drag-region" data-tauri-drag-region></div>

        <div class="nav-right no-drag">
          <a
            class="nav-icon-btn"
            href="https://github.com/Heavrnl/nexus-terminal"
            target="_blank"
            rel="noopener noreferrer"
            title="Heavrnl/nexus-terminal"
          >
            <i class="fab fa-github"></i>
          </a>
          <button v-if="isAuthenticated" @click="handleLogout" class="nav-link nav-link-ghost">登出</button>

          <div class="window-controls">
            <button class="window-btn minimize" @click="minimizeWindow" title="最小化" aria-label="最小化">
              <svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <path fill-rule="evenodd" d="M2 8a.5.5 0 0 1 .5-.5h11a.5.5 0 0 1 0 1h-11A.5.5 0 0 1 2 8Z" />
              </svg>
            </button>
            <button class="window-btn maximize" @click="toggleMaximize" title="最大化/还原" aria-label="最大化或还原">
              <svg viewBox="0 0 16 16" fill="currentColor" aria-hidden="true">
                <path d="M14 1a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1h12zM2 0a2 2 0 0 0-2 2v12a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V2a2 2 0 0 0-2-2H2z" />
              </svg>
            </button>
            <button class="window-btn close" @click="closeWindow" title="关闭" aria-label="关闭窗口">
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
      <span v-if="startupState === 'starting'">后端启动中...</span>
      <span v-else>后端启动失败：{{ startupError || '未知错误' }}</span>
    </div>

    <main class="app-main">
      <router-view />
    </main>

    <FocusSwitcherConfigurator
      :visible="isFocusSwitcherVisible"
      @close="focusSwitcherStore.toggleConfigurator(false)"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAuthStore } from '@/stores/auth';
import { useFocusSwitcherStore } from '@/stores/focusSwitcher';
import { statusApi } from '@/lib/api';
import UINotificationDisplay from '@/components/UINotificationDisplay.vue';
import GlobalAlertDialog from '@/components/GlobalAlertDialog.vue';
import GlobalConfirmDialog from '@/components/GlobalConfirmDialog.vue';
import FocusSwitcherConfigurator from '@/components/FocusSwitcherConfigurator.vue';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const focusSwitcherStore = useFocusSwitcherStore();
const { isAuthenticated } = storeToRefs(authStore);
const { isConfiguratorVisible: isFocusSwitcherVisible } = storeToRefs(focusSwitcherStore);
const appWindow = getCurrentWindow();

const noHeaderPaths = ['/login', '/setup'];
const showHeader = computed(() => !noHeaderPaths.includes(route.path));
const isWorkspaceRoute = computed(() => route.path === '/workspace');

const startupState = ref<'starting' | 'ready' | 'error'>('starting');
const startupError = ref('');

const isAltPressed = ref(false);
const altShortcutKey = ref<string | null>(null);
const lastFocusedIdBySwitcher = ref<string | null>(null);

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
  } catch (e: any) {
    startupState.value = 'error';
    startupError.value = e.message ?? String(e);
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

async function handleLogout() {
  await authStore.logout();
  router.push('/login');
}

function preventBrowserContextMenu(event: MouseEvent): void {
  event.preventDefault();
}

onMounted(() => {
  void checkBackendStartup();
  void focusSwitcherStore.loadConfigurationFromBackend();
  window.addEventListener('keydown', handleAltKeyDown);
  window.addEventListener('keyup', handleAltKeyUp);
  window.addEventListener('contextmenu', preventBrowserContextMenu);
});

onUnmounted(() => {
  window.removeEventListener('keydown', handleAltKeyDown);
  window.removeEventListener('keyup', handleAltKeyUp);
  window.removeEventListener('contextmenu', preventBrowserContextMenu);
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
  height: 52px;
  background: var(--bg-surface0);
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

.brand-mark {
  width: 26px;
  height: 26px;
  margin-right: 4px;
  border-radius: 6px;
  background: linear-gradient(145deg, rgba(255, 255, 255, 0.18), rgba(137, 180, 250, 0.18));
  display: inline-flex;
  align-items: center;
  justify-content: center;
  color: #d9e2ff;
  font-size: 15px;
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
  font-size: 13px;
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
  background: rgba(137, 180, 250, 0.12);
}

.nav-link-ghost {
  padding: 0 10px;
}

.nav-icon-btn {
  width: 32px;
  height: 32px;
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

.window-controls {
  display: flex;
  align-items: center;
  margin-left: 4px;
  border-left: 1px solid rgba(205, 214, 244, 0.14);
  padding-left: 6px;
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
  color: #fff;
  background: #dc2626;
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
  outline: 1px solid rgba(137, 180, 250, 0.8);
  outline-offset: 1px;
}

.no-drag {
  -webkit-app-region: no-drag;
}

.startup-banner {
  padding: 6px 12px;
  font-size: 12px;
  border-bottom: 1px solid var(--border);
}

.startup-banner.starting {
  background: rgba(59, 130, 246, 0.12);
  color: #60a5fa;
}

.startup-banner.error {
  background: rgba(220, 38, 38, 0.14);
  color: #f87171;
}

.app-main {
  flex: 1;
  overflow: hidden;
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
    font-size: 12px;
  }

  .nav-link-desktop {
    display: none;
  }
}
</style>
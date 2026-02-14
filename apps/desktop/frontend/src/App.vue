<template>
  <UINotificationDisplay />
  <GlobalAlertDialog />
  <GlobalConfirmDialog />

  <div id="app-container">
    <!-- 顶部导航栏 -->
    <header v-if="showHeader" class="app-header">
      <nav class="app-nav">
        <!-- 左侧导航链接 -->
        <div class="nav-left">
          <router-link to="/connections" class="nav-link" active-class="nav-link-active">连接管理</router-link>
          <router-link to="/workspace" class="nav-link" active-class="nav-link-active">工作区</router-link>
          <router-link to="/proxies" class="nav-link" active-class="nav-link-active">代理</router-link>
          <router-link to="/settings" class="nav-link" active-class="nav-link-active">设置</router-link>
        </div>

        <div class="nav-drag-region" data-tauri-drag-region @mousedown.left="startDrag"></div>

        <!-- 右侧操作按钮 -->
        <div class="nav-right">
          <button v-if="isAuthenticated" @click="handleLogout" class="nav-link no-drag">退出</button>
          <div class="window-controls no-drag">
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

    <!-- 主内容区 -->
    <main class="app-main">
      <router-view />
    </main>
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import { storeToRefs } from 'pinia';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { useAuthStore } from '@/stores/auth';
import { statusApi } from '@/lib/api';
import UINotificationDisplay from '@/components/UINotificationDisplay.vue';
import GlobalAlertDialog from '@/components/GlobalAlertDialog.vue';
import GlobalConfirmDialog from '@/components/GlobalConfirmDialog.vue';

const route = useRoute();
const router = useRouter();
const authStore = useAuthStore();
const { isAuthenticated } = storeToRefs(authStore);
const appWindow = getCurrentWindow();

const noHeaderPaths = ['/login', '/setup'];
const showHeader = computed(() => !noHeaderPaths.includes(route.path));

const startupState = ref<'starting' | 'ready' | 'error'>('starting');
const startupError = ref('');

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

async function minimizeWindow() {
  await appWindow.minimize();
}

async function toggleMaximize() {
  await appWindow.toggleMaximize();
}

async function closeWindow() {
  await appWindow.close();
}

async function startDrag() {
  try {
    await appWindow.startDragging();
  } catch {
    // ignore drag failures (e.g. unsupported platform/window state)
  }
}

async function handleLogout() {
  await authStore.logout();
  router.push('/login');
}

onMounted(() => {
  void checkBackendStartup();
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
  position: sticky;
  top: 0;
  z-index: 10;
  height: 56px;
  background: var(--bg-surface0);
  border-bottom: 1px solid var(--border);
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.app-nav {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 100%;
  padding: 0 16px;
}

.nav-left, .nav-right {
  display: flex;
  align-items: center;
  gap: 4px;
}

.nav-drag-region {
  flex: 1;
  height: 100%;
}

.nav-link {
  display: inline-flex;
  align-items: center;
  padding: 8px 12px;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 500;
  color: var(--subtext0);
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
  background: rgba(137, 180, 250, 0.1);
}

.window-btn {
  width: 30px;
  height: 24px;
  border: none;
  border-radius: 4px;
  background: transparent;
  color: var(--text-sub);
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s ease;
}

.window-controls {
  display: flex;
  align-items: center;
  gap: 2px;
  margin-left: 6px;
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
  width: 16px;
  height: 16px;
  flex-shrink: 0;
}

.window-btn.maximize svg {
  width: 13px;
  height: 13px;
}

.window-btn:focus-visible {
  outline: 1px solid color-mix(in srgb, var(--blue) 70%, transparent);
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
</style>

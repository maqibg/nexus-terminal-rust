<template>
  <nav class="nav-sidebar">
    <div class="nav-brand">Nexus Terminal</div>
    <div class="nav-links">
      <router-link v-for="item in navItems" :key="item.path" :to="item.path" class="nav-item" active-class="nav-active">
        {{ item.label }}
      </router-link>
    </div>
    <div class="nav-footer">
      <button class="nav-item nav-logout" @click="logout">登出</button>
    </div>
  </nav>
</template>

<script setup lang="ts">
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const router = useRouter();
const auth = useAuthStore();

const navItems = [
  { path: '/', label: '仪表盘' },
  { path: '/workspace', label: '终端' },
  { path: '/connections', label: '连接管理' },
  { path: '/proxies', label: '代理管理' },
  { path: '/tags', label: '标签' },
  { path: '/quick-commands', label: '快捷命令' },
  { path: '/command-history', label: '命令历史' },
  { path: '/notifications', label: '通知' },
  { path: '/suspended-sessions', label: '挂起会话' },
  { path: '/settings', label: '设置' },
  { path: '/audit', label: '审计日志' },
];

async function logout() {
  await auth.logout();
  router.push('/login');
}
</script>

<style scoped>
.nav-sidebar {
  width: 180px;
  height: 100%;
  background: var(--bg-mantle);
  border-right: 1px solid var(--border);
  display: flex;
  flex-direction: column;
  flex-shrink: 0;
}
.nav-brand {
  padding: 16px 14px;
  font-size: 14px;
  font-weight: 700;
  color: var(--blue);
  border-bottom: 1px solid var(--border);
}
.nav-links {
  flex: 1;
  overflow-y: auto;
  padding: 8px 0;
}
.nav-item {
  display: block;
  padding: 8px 14px;
  font-size: 13px;
  color: var(--text-sub);
  text-decoration: none;
  border: none;
  background: none;
  width: 100%;
  text-align: left;
  cursor: pointer;
}
.nav-item:hover { color: var(--text); background: var(--bg-surface0); }
.nav-active { color: var(--blue); background: var(--bg-surface0); }
.nav-footer {
  border-top: 1px solid var(--border);
  padding: 8px 0;
}
.nav-logout { color: var(--red); }
.nav-logout:hover { background: var(--bg-surface0); }
</style>

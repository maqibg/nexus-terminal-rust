import { createApp } from 'vue';
import { createPinia } from 'pinia';
import { createRouter, createWebHashHistory } from 'vue-router';
import App from './App.vue';
import { useAuthStore } from './stores/auth';
import '@fortawesome/fontawesome-free/css/all.min.css';
import './assets/global.css';

const router = createRouter({
  history: createWebHashHistory(),
  routes: [
    { path: '/login', component: () => import('./views/Login.vue') },
    { path: '/setup', component: () => import('./views/Setup.vue') },
    { path: '/', redirect: '/connections' },
    { path: '/connections', component: () => import('./views/Connections.vue'), meta: { requiresAuth: true } },
    { path: '/workspace', component: () => import('./views/Workspace.vue'), meta: { requiresAuth: true } },
    { path: '/databases', component: () => import('./views/Databases.vue'), meta: { requiresAuth: true } },
    { path: '/tools', component: () => import('./views/Tools.vue'), meta: { requiresAuth: true } },
    { path: '/proxies', component: () => import('./views/Proxies.vue'), meta: { requiresAuth: true } },
    { path: '/statistics', component: () => import('./views/Statistics.vue'), meta: { requiresAuth: true } },
    { path: '/settings', component: () => import('./views/Settings.vue'), meta: { requiresAuth: true } },
  ],
});

const app = createApp(App);
const pinia = createPinia();
app.use(pinia);

// Auth guard
router.beforeEach(async (to) => {
  const auth = useAuthStore();
  if (auth.state === 'loading') await auth.checkStatus();

  // Setup page — no auth required, redirect if not needed
  if (to.path === '/setup') {
    if (auth.state === 'NeedsSetup') return true;
    return '/';
  }

  if (to.path === '/login') {
    if (auth.state === 'Authenticated') return '/';
    return true;
  }

  if (auth.state === 'NeedsSetup') return '/setup';
  if (auth.state !== 'Authenticated') return '/login';
  return true;
});

app.use(router);
app.mount('#app');

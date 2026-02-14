import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import { authApi } from '@/lib/api';

export const useAuthStore = defineStore('auth', () => {
  const state = ref<string>('loading');
  const has2fa = ref(false);
  const isAuthenticated = computed(() => state.value === 'Authenticated');

  async function checkStatus() {
    try {
      const s = await authApi.status();
      state.value = s.state;
      has2fa.value = s.has_2fa;
    } catch {
      state.value = 'error';
    }
  }

  async function setup(username: string, password: string) {
    await authApi.setup(username, password);
    await checkStatus();
  }

  async function login(username: string, password: string): Promise<boolean> {
    const r = await authApi.login(username, password);
    if (r.needs_2fa) return true;
    await checkStatus();
    return false;
  }

  async function verify2fa(token: string) {
    await authApi.verify2fa(token);
    await checkStatus();
  }

  async function logout() {
    await authApi.logout();
    state.value = 'Locked';
  }

  return { state, has2fa, isAuthenticated, checkStatus, setup, login, verify2fa, logout };
});

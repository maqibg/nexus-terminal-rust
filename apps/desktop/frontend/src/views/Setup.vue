<template>
  <div class="setup-page">
    <div class="setup-card">
      <h2>初始设置</h2>
      <p class="setup-desc">创建管理员账户以开始使用</p>
      <form @submit.prevent="handleSetup">
        <label class="field">
          <span class="label">用户名</span>
          <input v-model="username" class="input" required />
        </label>
        <label class="field">
          <span class="label">密码</span>
          <input v-model="password" type="password" class="input" required />
        </label>
        <label class="field">
          <span class="label">确认密码</span>
          <input v-model="confirmPassword" type="password" class="input" required />
        </label>
        <div v-if="error" class="error">{{ error }}</div>
        <button class="btn btn-primary" type="submit" :disabled="loading">
          {{ loading ? '创建中...' : '创建账户' }}
        </button>
      </form>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const router = useRouter();
const auth = useAuthStore();
const username = ref('');
const password = ref('');
const confirmPassword = ref('');
const error = ref('');
const loading = ref(false);

async function handleSetup() {
  if (password.value !== confirmPassword.value) { error.value = '密码不一致'; return; }
  if (password.value.length < 6) { error.value = '密码至少 6 位'; return; }
  error.value = '';
  loading.value = true;
  try {
    await auth.setup(username.value, password.value);
    router.push('/login');
  } catch (e: any) {
    error.value = e.message ?? String(e);
  } finally {
    loading.value = false;
  }
}
</script>

<style scoped>
.setup-page { display: flex; align-items: center; justify-content: center; height: 100vh; background: var(--bg-base); }
.setup-card { background: var(--bg-surface0); border: 1px solid var(--border); border-radius: 8px; padding: 32px; width: 360px; }
.setup-card h2 { margin-bottom: 4px; color: var(--text); font-size: 18px; }
.setup-desc { color: var(--text-dim); font-size: 13px; margin-bottom: 20px; }
.field { display: flex; flex-direction: column; gap: 4px; margin-bottom: 12px; }
.label { font-size: 12px; color: var(--text-sub); }
.input { padding: 8px 10px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 4px; color: var(--text); font-size: 13px; }
.input:focus { outline: none; border-color: var(--blue); }
.error { color: var(--red); font-size: 12px; margin-bottom: 8px; }
.btn-primary { width: 100%; padding: 8px; background: var(--blue); color: var(--bg-base); border: none; border-radius: 4px; cursor: pointer; font-size: 14px; }
.btn-primary:disabled { opacity: 0.5; }
</style>

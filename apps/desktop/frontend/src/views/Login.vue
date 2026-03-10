<template>
  <div class="auth-page">
    <div class="auth-card">
      <h1>Nexus Terminal</h1>
      <p class="subtitle">{{ isSetup ? '创建管理员账户' : '登录' }}</p>

      <div v-if="error" class="error">{{ error }}</div>

      <!-- 2FA verification -->
      <template v-if="needs2fa">
        <input v-model="token" placeholder="2FA 验证码" @keyup.enter="handleVerify2fa" />
        <button @click="handleVerify2fa" :disabled="busy">验证</button>
      </template>

      <!-- Login / Setup -->
      <template v-else>
        <input v-model="username" placeholder="用户名" @keyup.enter="handleSubmit" />
        <input v-model="password" type="password" placeholder="密码" @keyup.enter="handleSubmit" />
        <input v-if="isSetup" v-model="confirmPassword" type="password" placeholder="确认密码" @keyup.enter="handleSubmit" />
        <button @click="handleSubmit" :disabled="busy">
          {{ isSetup ? '创建' : '登录' }}
        </button>
      </template>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useRouter } from 'vue-router';
import { useAuthStore } from '@/stores/auth';

const auth = useAuthStore();
const router = useRouter();

const username = ref('');
const password = ref('');
const confirmPassword = ref('');
const token = ref('');
const error = ref('');
const busy = ref(false);
const needs2fa = ref(false);

const isSetup = computed(() => auth.state === 'NeedsSetup');

async function handleSubmit() {
  error.value = '';
  if (!username.value || !password.value) { error.value = '请填写所有字段'; return; }
  if (isSetup.value && password.value !== confirmPassword.value) { error.value = '密码不一致'; return; }

  busy.value = true;
  try {
    if (isSetup.value) {
      await auth.setup(username.value, password.value);
    } else {
      const need2fa = await auth.login(username.value, password.value);
      if (need2fa) { needs2fa.value = true; return; }
    }
    router.push('/');
  } catch (e: any) {
    error.value = e.message;
  } finally {
    busy.value = false;
  }
}

async function handleVerify2fa() {
  error.value = '';
  busy.value = true;
  try {
    await auth.verify2fa(token.value);
    router.push('/');
  } catch (e: any) {
    error.value = e.message;
  } finally {
    busy.value = false;
  }
}
</script>

<style scoped>
.auth-page {
  display: flex; align-items: center; justify-content: center;
  height: 100%; background: var(--bg-base);
}
.auth-card {
  display: flex; flex-direction: column; gap: 0.75rem;
  padding: 2rem;
  border-radius: 12px;
  border: 1px solid var(--border);
  background: var(--bg-surface0);
  min-width: 320px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.3);
}
h1 { text-align: center; font-weight: 300; color: var(--text); margin: 0; }
.subtitle { text-align: center; color: var(--text-sub); font-size: calc(0.9rem + var(--ui-font-size-offset)); margin: 0; }
input {
  padding: 0.6rem 0.8rem;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: calc(0.9rem + var(--ui-font-size-offset));
  outline: none;
}
input:focus {
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}
button {
  padding: 0.6rem;
  border-radius: 6px;
  border: none;
  cursor: pointer;
  background: var(--button-bg-color);
  color: var(--button-text-color);
  font-weight: 600;
  font-size: calc(0.9rem + var(--ui-font-size-offset));
  transition: background 0.15s ease;
}
button:hover { background: var(--button-hover-bg-color); }
button:disabled { opacity: 0.5; cursor: not-allowed; }
.error { color: var(--red); font-size: calc(0.85rem + var(--ui-font-size-offset)); text-align: center; }
</style>

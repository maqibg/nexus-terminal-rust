<template>
  <section class="passkey-section">
    <h3 class="section-title">Passkey 管理</h3>
    <div class="key-list">
      <div v-for="pk in passkeys" :key="pk.credential_id" class="key-item">
        <span class="key-name">{{ pk.name }}</span>
        <div class="key-actions">
          <button class="btn-sm" @click="rename(pk)">重命名</button>
          <button class="btn-sm danger" @click="remove(pk)">删除</button>
        </div>
      </div>
      <div v-if="!passkeys.length" class="empty">暂无 Passkey</div>
    </div>
    <button class="btn-add" :disabled="registering" @click="register">
      {{ registering ? '注册中...' : '注册新 Passkey' }}
    </button>
  </section>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { authApi, type PasskeyInfo } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const notify = useUINotificationStore();
const passkeys = ref<PasskeyInfo[]>([]);
const registering = ref(false);

async function load() {
  try { passkeys.value = await authApi.passkeyList(); } catch { passkeys.value = []; }
}
onMounted(load);

/** Convert ArrayBuffer to base64url string (no padding). */
function bufferToBase64url(buf: ArrayBuffer): string {
  const bytes = new Uint8Array(buf);
  let str = '';
  for (const b of bytes) str += String.fromCharCode(b);
  return btoa(str).replace(/\+/g, '-').replace(/\//g, '_').replace(/=/g, '');
}

/** Decode base64url string to Uint8Array. */
function base64urlToUint8Array(s: string): Uint8Array<ArrayBuffer> {
  const padded = s.replace(/-/g, '+').replace(/_/g, '/').padEnd(s.length + (4 - (s.length % 4)) % 4, '=');
  const binary = atob(padded);
  const bytes = new Uint8Array(new ArrayBuffer(binary.length));
  for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
  return bytes;
}

/**
 * Serialize a PublicKeyCredential (from navigator.credentials.create) to the JSON format
 * expected by webauthn-rs's RegisterPublicKeyCredential.
 */
function serializeCredential(credential: PublicKeyCredential): string {
  const response = credential.response as AuthenticatorAttestationResponse;
  const transports = response.getTransports ? response.getTransports() : [];
  return JSON.stringify({
    id: credential.id,
    rawId: bufferToBase64url(credential.rawId),
    type: credential.type,
    response: {
      clientDataJSON: bufferToBase64url(response.clientDataJSON),
      attestationObject: bufferToBase64url(response.attestationObject),
      transports,
    },
  });
}

async function register() {
  const name = prompt('Passkey 名称:');
  if (!name?.trim()) return;

  registering.value = true;
  try {
    // 1. Get creation options from server (CreationChallengeResponse JSON)
    const optionsJson = await authApi.passkeyRegisterStart();
    const options = JSON.parse(optionsJson) as { publicKey: PublicKeyCredentialCreationOptions };

    // Convert base64url-encoded fields to ArrayBuffer as required by the WebAuthn API
    const pubKeyOpts: PublicKeyCredentialCreationOptions = {
      ...options.publicKey,
      challenge: base64urlToUint8Array(options.publicKey.challenge as unknown as string),
      user: {
        ...options.publicKey.user,
        id: base64urlToUint8Array(options.publicKey.user.id as unknown as string),
      },
    };

    // 2. Invoke the browser authenticator
    const credential = await navigator.credentials.create({ publicKey: pubKeyOpts });
    if (!credential || !(credential instanceof PublicKeyCredential)) {
      throw new Error('authenticator returned no credential');
    }

    // 3. Serialize and send to server for verification and storage
    const credentialJson = serializeCredential(credential);
    await authApi.passkeyRegisterFinish(credentialJson, name.trim());

    notify.addNotification('success', 'Passkey 已注册');
    load();
  } catch (e: unknown) {
    const msg = e instanceof Error ? e.message : String(e);
    notify.addNotification('error', `注册失败: ${msg}`);
  } finally {
    registering.value = false;
  }
}

async function remove(pk: PasskeyInfo) {
  if (!confirm(`确定删除 "${pk.name}"？`)) return;
  try {
    await authApi.passkeyDelete(pk.credential_id);
    notify.addNotification('success', 'Passkey 已删除');
    load();
  } catch (e: unknown) {
    notify.addNotification('error', e instanceof Error ? e.message : String(e));
  }
}

async function rename(pk: PasskeyInfo) {
  const name = prompt('新名称:', pk.name);
  if (!name || name === pk.name) return;
  try {
    await authApi.passkeyRename(pk.credential_id, name);
    notify.addNotification('success', '已重命名');
    load();
  } catch (e: unknown) {
    notify.addNotification('error', e instanceof Error ? e.message : String(e));
  }
}
</script>

<style scoped>
.passkey-section { display: flex; flex-direction: column; gap: 10px; }
.section-title { font-size: calc(15px + var(--ui-font-size-offset)); font-weight: 600; margin: 0; padding-bottom: 8px; border-bottom: 1px solid var(--border); }
.key-list { display: flex; flex-direction: column; gap: 4px; }
.key-item { display: flex; justify-content: space-between; align-items: center; padding: 8px; border-radius: 4px; background: var(--bg-mantle); }
.key-name { font-size: calc(13px + var(--ui-font-size-offset)); }
.key-actions { display: flex; gap: 4px; }
.btn-sm { padding: 3px 10px; border-radius: 3px; border: 1px solid var(--border); background: transparent; color: var(--text); cursor: pointer; font-size: calc(12px + var(--ui-font-size-offset)); }
.btn-sm:hover { background: var(--bg-surface1); }
.btn-sm.danger { color: var(--red); border-color: var(--red); }
.btn-sm.danger:hover { background: rgba(243,139,168,0.1); }
.btn-add { align-self: flex-start; padding: 5px 14px; border-radius: 4px; border: none; background: var(--blue); color: var(--bg-base); cursor: pointer; font-size: calc(13px + var(--ui-font-size-offset)); font-weight: 600; }
.btn-add:hover:not(:disabled) { opacity: 0.9; }
.btn-add:disabled { opacity: 0.5; cursor: not-allowed; }
.empty { text-align: center; color: var(--text-dim); font-size: calc(13px + var(--ui-font-size-offset)); padding: 12px; }
</style>

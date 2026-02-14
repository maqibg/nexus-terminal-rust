<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="$emit('cancel')">
      <div class="form-card">
        <div class="form-title">{{ channel ? '编辑通知渠道' : '新建通知渠道' }}</div>
        <div class="form-body">
          <input v-model="form.name" class="input" placeholder="名称" />
          <select v-model="form.channel_type" class="input">
            <option value="webhook">Webhook</option>
            <option value="email">Email</option>
            <option value="telegram">Telegram</option>
          </select>
          <template v-if="form.channel_type === 'webhook'">
            <input v-model="config.url" class="input" placeholder="Webhook URL" />
          </template>
          <template v-else-if="form.channel_type === 'email'">
            <input v-model="config.smtp_host" class="input" placeholder="SMTP 主机" />
            <input v-model.number="config.smtp_port" class="input" type="number" placeholder="SMTP 端口" />
            <input v-model="config.smtp_user" class="input" placeholder="用户名" />
            <input v-model="config.smtp_pass" class="input" type="password" placeholder="密码" />
            <input v-model="config.to" class="input" placeholder="收件人" />
          </template>
          <template v-else-if="form.channel_type === 'telegram'">
            <input v-model="config.bot_token" class="input" placeholder="Bot Token" />
            <input v-model="config.chat_id" class="input" placeholder="Chat ID" />
          </template>
          <label class="checkbox-row">
            <input type="checkbox" v-model="form.enabled" /> 启用
          </label>
        </div>
        <div class="form-actions">
          <button class="btn btn-cancel" @click="$emit('cancel')">取消</button>
          <button class="btn btn-primary" @click="submit" :disabled="!form.name">保存</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { reactive, watch } from 'vue';
import { settingsApi, type NotificationChannel } from '@/lib/api';
import { useUINotificationStore } from '@/stores/uiNotifications';

const props = defineProps<{ visible: boolean; channel?: NotificationChannel }>();
const emit = defineEmits<{ saved: []; cancel: [] }>();
const notify = useUINotificationStore();

const form = reactive({ name: '', channel_type: 'webhook', enabled: true });
const config = reactive<Record<string, any>>({});

watch(() => props.channel, (c) => {
  if (c) {
    Object.assign(form, { name: c.name, channel_type: c.channel_type, enabled: c.enabled });
    try { Object.assign(config, JSON.parse(c.config)); } catch { /* ignore */ }
  } else {
    Object.assign(form, { name: '', channel_type: 'webhook', enabled: true });
    Object.keys(config).forEach(k => delete config[k]);
  }
}, { immediate: true });

async function submit() {
  try {
    const data = {
      id: props.channel?.id ?? 0,
      name: form.name, channel_type: form.channel_type,
      enabled: form.enabled, config: JSON.stringify(config),
      enabled_events: props.channel?.enabled_events ?? '[]',
    };
    if (props.channel) await settingsApi.notificationChannelUpdate(data);
    else await settingsApi.notificationChannelCreate(data);
    notify.addNotification('success', props.channel ? '渠道已更新' : '渠道已创建');
    emit('saved');
  } catch (e: any) { notify.addNotification('error', e.message); }
}
</script>

<style scoped>
.dialog-backdrop { position: fixed; inset: 0; z-index: 9000; background: rgba(0,0,0,0.5); display: flex; align-items: center; justify-content: center; }
.form-card { background: var(--bg-surface0); border-radius: 8px; padding: 24px; width: 420px; border: 1px solid var(--border); }
.form-title { font-size: 16px; font-weight: 600; margin-bottom: 16px; }
.form-body { display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px; }
.input { background: var(--bg-mantle); border: 1px solid var(--border); border-radius: 4px; padding: 8px; color: var(--text); font-size: 13px; outline: none; }
.input:focus { border-color: var(--blue); }
.checkbox-row { display: flex; align-items: center; gap: 6px; font-size: 13px; color: var(--text-sub); cursor: pointer; }
.form-actions { display: flex; justify-content: flex-end; gap: 8px; }
.btn { padding: 6px 16px; border-radius: 4px; border: none; cursor: pointer; font-size: 13px; }
.btn-cancel { background: var(--bg-surface1); color: var(--text-sub); }
.btn-primary { background: var(--blue); color: var(--bg-base); }
.btn:disabled { opacity: 0.4; cursor: default; }
</style>

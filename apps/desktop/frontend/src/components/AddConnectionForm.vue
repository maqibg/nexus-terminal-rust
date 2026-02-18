<script setup lang="ts">
import { computed, toRef } from 'vue';
import { useAddConnectionForm } from '@/composables/useAddConnectionForm';
import AddConnectionFormBasicInfo from './AddConnectionFormBasicInfo.vue';
import AddConnectionFormAuth from './AddConnectionFormAuth.vue';
import AddConnectionFormAdvanced from './AddConnectionFormAdvanced.vue';

const props = defineProps<{
  visible: boolean;
  mode: 'create' | 'edit';
  connectionId?: number;
}>();

const emit = defineEmits<{
  (e: 'close'): void;
  (e: 'saved'): void;
}>();

const form = useAddConnectionForm(
  toRef(props, 'mode'),
  toRef(props, 'connectionId'),
  toRef(props, 'visible'),
  () => emit('saved'),
  () => emit('close'),
);

const isActionDisabled = computed(() => {
  if (form.isLoading.value) {
    return true;
  }
  return form.formData.type === 'SSH' && form.testStatus.value === 'testing';
});

const toggleScriptMode = () => {
  form.isScriptModeActive.value = !form.isScriptModeActive.value;
};

const handleAdvancedConnectionModeUpdate = (newMode: 'proxy' | 'jump') => {
  form.advancedConnectionMode.value = newMode;
};
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="dialog-backdrop" @click.self="emit('close')">
      <div class="dialog-panel">
        <h3 class="dialog-title">{{ form.formTitle.value }}</h3>

        <form id="add-connection-form" class="dialog-form" @submit.prevent="form.handleSubmit">
          <template v-if="!form.isScriptModeActive.value">
            <AddConnectionFormBasicInfo :form-data="form.formData" />
            <AddConnectionFormAuth :form-data="form.formData" :is-edit-mode="form.isEditMode.value" />
            <AddConnectionFormAdvanced
              :form-data="form.formData"
              :proxies="form.proxies.value"
              :tags="form.tags.value"
              :connections="form.connections.value"
              :is-proxy-loading="form.isProxyLoading.value"
              :proxy-store-error="form.proxyStoreError.value"
              :is-tag-loading="form.isTagLoading.value"
              :tag-store-error="form.tagStoreError.value"
              :advanced-connection-mode="form.advancedConnectionMode.value"
              :is-edit-mode="form.isEditMode.value"
              :add-jump-host="form.addJumpHost"
              :remove-jump-host="form.removeJumpHost"
              @update:advancedConnectionMode="handleAdvancedConnectionModeUpdate"
              @create-tag="form.handleCreateTag"
              @delete-tag="form.handleDeleteTag"
            />
          </template>

          <div v-if="!form.isEditMode.value" class="section-card script-mode-card">
            <div class="script-mode-header">
              <h4 class="section-title script-mode-title">脚本模式</h4>
              <button
                type="button"
                class="script-mode-switch"
                :class="{ active: form.isScriptModeActive.value }"
                role="switch"
                :aria-checked="form.isScriptModeActive.value"
                @click="toggleScriptMode"
              >
                <span class="script-mode-switch-knob" :class="{ active: form.isScriptModeActive.value }"></span>
              </button>
            </div>

            <div v-if="form.isScriptModeActive.value" class="script-mode-body">
              <textarea
                id="conn-script-input"
                v-model="form.scriptInputText.value"
                rows="10"
                wrap="off"
                class="script-mode-textarea"
                placeholder="每行一个连接，示例：root@192.168.1.10:22 -p 123456"
              ></textarea>
              <p class="script-mode-help">{{ form.scriptModeFormatInfo }}</p>
            </div>
          </div>
        </form>

        <div class="dialog-actions">
          <div v-if="form.formData.type === 'SSH' && !form.isScriptModeActive.value" class="test-area">
            <div class="test-action-row">
              <button
                type="button"
                class="test-button"
                :disabled="form.isLoading.value || form.testStatus.value === 'testing'"
                @click="form.handleTestConnection"
              >
                <i v-if="form.testStatus.value === 'testing'" class="fas fa-spinner fa-spin"></i>
                <span>{{ form.testButtonText.value }}</span>
              </button>

              <div class="test-tip-wrap">
                <i class="fas fa-info-circle"></i>
                <span class="test-tip-text">网络波动会导致延迟上下浮动，建议多次测试取平均值。</span>
              </div>
            </div>

            <div class="test-result-row">
              <span v-if="form.testStatus.value === 'testing'" class="test-result testing">测试中...</span>
              <span v-else-if="form.testStatus.value === 'success'" class="test-result" :style="{ color: form.latencyColor.value }">{{ form.testResult.value }}</span>
              <span v-else-if="form.testStatus.value === 'error'" class="test-result error">错误: {{ form.testResult.value }}</span>
            </div>
          </div>
          <div v-else class="test-area-placeholder"></div>

          <div class="main-actions">
            <button
              v-if="form.isEditMode.value && !form.isScriptModeActive.value"
              type="button"
              class="btn btn-danger"
              :disabled="isActionDisabled"
              @click="form.handleDeleteConnection"
            >
              删除
            </button>

            <button type="submit" form="add-connection-form" class="btn btn-primary" :disabled="isActionDisabled">{{ form.submitButtonText.value }}</button>
            <button type="button" class="btn btn-secondary" :disabled="isActionDisabled" @click="emit('close')">取消</button>
          </div>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.dialog-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 16px;
  background: var(--ui-overlay);
}

.dialog-panel {
  width: min(100%, 840px);
  max-height: 90vh;
  display: flex;
  flex-direction: column;
  padding: 24px;
  border: 1px solid var(--ui-dialog-border);
  border-radius: 10px;
  background: var(--ui-dialog-bg);
  box-shadow: 0 24px 64px var(--ui-dialog-shadow);
}

.dialog-title {
  margin: 0 0 16px;
  text-align: center;
  font-size: 20px;
  font-weight: 600;
  color: var(--ui-text-strong);
}

.dialog-form {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 14px;
  overflow-y: auto;
  padding-right: 4px;
}

.section-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface1) 45%, transparent);
}

.section-title {
  margin: 0;
  font-size: 15px;
  font-weight: 600;
  color: var(--text);
}

.script-mode-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.script-mode-switch {
  position: relative;
  width: 44px;
  height: 24px;
  border: none;
  border-radius: 99px;
  background: #5d6570;
  cursor: pointer;
}

.script-mode-switch.active {
  background: var(--blue);
}

.script-mode-switch-knob {
  position: absolute;
  top: 2px;
  left: 2px;
  width: 20px;
  height: 20px;
  border-radius: 50%;
  background: #ffffff;
  transition: transform 0.18s ease;
}

.script-mode-switch-knob.active {
  transform: translateX(20px);
}

.script-mode-textarea {
  width: 100%;
  min-height: 190px;
  resize: vertical;
  padding: 10px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: 13px;
  line-height: 1.5;
  font-family: 'Consolas', 'Courier New', monospace;
}

.script-mode-textarea:focus {
  outline: none;
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.script-mode-help {
  margin: 8px 0 0;
  font-size: 12px;
  line-height: 1.5;
  color: var(--text-dim);
  white-space: pre-line;
}

.dialog-actions {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid var(--ui-divider);
  display: flex;
  align-items: flex-end;
  justify-content: space-between;
  gap: 12px;
  flex-wrap: wrap;
}

.test-area { min-height: 56px; display: flex; flex-direction: column; gap: 6px; }
.test-action-row { display: flex; align-items: center; gap: 8px; }

.test-button {
  height: 32px;
  padding: 0 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text-sub);
  font-size: 13px;
  display: inline-flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
}

.test-button:hover { background: var(--bg-surface1); color: var(--text); }
.test-button:disabled { opacity: 0.55; cursor: not-allowed; }

.test-tip-wrap { position: relative; color: var(--text-sub); font-size: 14px; cursor: help; }

.test-tip-text {
  position: absolute;
  left: 50%;
  bottom: calc(100% + 6px);
  transform: translateX(-50%);
  width: max-content;
  max-width: 240px;
  padding: 6px 8px;
  border-radius: 6px;
  background: #1f2430;
  color: #ffffff;
  font-size: 12px;
  line-height: 1.45;
  opacity: 0;
  pointer-events: none;
  transition: opacity 0.15s ease;
}

.test-tip-wrap:hover .test-tip-text { opacity: 1; }
.test-result-row { min-height: 18px; }
.test-result { font-size: 12px; font-weight: 500; }
.test-result.testing { color: var(--text-sub); }
.test-result.error { color: var(--red); }
.test-area-placeholder { flex: 1; }

.main-actions { margin-left: auto; display: flex; align-items: center; gap: 8px; }

.btn {
  height: 34px;
  padding: 0 14px;
  border-radius: 6px;
  border: 1px solid transparent;
  font-size: 13px;
  cursor: pointer;
}

.btn:disabled { opacity: 0.55; cursor: not-allowed; }
.btn-danger { border-color: var(--red); color: var(--red); background: transparent; }
.btn-danger:hover { background: color-mix(in srgb, var(--red) 12%, transparent); }
.btn-primary { border-color: var(--blue); background: var(--blue); color: #ffffff; }
.btn-primary:hover { filter: brightness(1.05); }
.btn-secondary { border-color: var(--border); background: transparent; color: var(--text-sub); }
.btn-secondary:hover { background: var(--bg-surface1); color: var(--text); }

@media (max-width: 920px) {
  .dialog-panel { width: min(100%, 760px); }
  .test-area,
  .test-area-placeholder,
  .main-actions { width: 100%; margin-left: 0; }
  .main-actions { justify-content: flex-end; }
}
</style>

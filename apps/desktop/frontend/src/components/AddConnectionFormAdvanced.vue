<script setup lang="ts">
import { type PropType } from 'vue';
import TagInput from './TagInput.vue';

interface ProxyInfo {
  id: number;
  name: string;
  proxy_type: string;
  host: string;
  port: number;
}

interface TagInfo {
  id: number;
  name: string;
}

interface ConnectionInfo {
  id: number;
  name: string;
  type: string;
}

const props = defineProps({
  formData: {
    type: Object as PropType<{
      id?: number;
      type: 'SSH' | 'RDP' | 'VNC';
      proxy_id: number | null;
      jump_chain: Array<number | null> | null;
      tag_ids: number[];
      notes: string;
      rdpOptions: {
        width: number | null;
        height: number | null;
        fullscreen: boolean;
        multimon: boolean;
        admin: boolean;
        restrictedAdmin: boolean;
        remoteGuard: boolean;
        drives: boolean;
        printers: boolean;
        clipboard: boolean;
        audio: 'local' | 'remote' | 'none';
        colorDepth: 15 | 16 | 24 | 32;
        compression: boolean;
        gateway: {
          enabled: boolean;
          host: string;
          port: number | null;
          username: string;
          password: string;
        };
      };
      vncOptions: {
        viewOnly: boolean;
        quality: number;
        compression: number;
        localCursor: boolean;
        sharedConnection: boolean;
      };
    }>,
    required: true,
  },
  proxies: { type: Array as PropType<ProxyInfo[]>, required: true },
  connections: { type: Array as PropType<ConnectionInfo[]>, required: true },
  tags: { type: Array as PropType<TagInfo[]>, required: true },
  isProxyLoading: { type: Boolean, required: true },
  proxyStoreError: { type: String as PropType<string | null>, default: null },
  isTagLoading: { type: Boolean, required: true },
  tagStoreError: { type: String as PropType<string | null>, default: null },
  advancedConnectionMode: { type: String as PropType<'proxy' | 'jump'>, required: true },
  addJumpHost: { type: Function as PropType<() => void>, required: true },
  removeJumpHost: { type: Function as PropType<(index: number) => void>, required: true },
  isEditMode: { type: Boolean, default: false },
});

const emit = defineEmits<{
  (e: 'create-tag', tagName: string): void;
  (e: 'delete-tag', tagId: number): void;
  (e: 'update:advancedConnectionMode', mode: 'proxy' | 'jump'): void;
}>();

const setConnectionMode = (mode: 'proxy' | 'jump') => {
  if (props.advancedConnectionMode === mode) {
    return;
  }
  emit('update:advancedConnectionMode', mode);
};

const getAvailableJumpHostsForIndex = (currentIndex: number): ConnectionInfo[] => {
  return props.connections.filter((conn) => {
    if (String(conn.type).toUpperCase() !== 'SSH') {
      return false;
    }
    if (props.isEditMode && props.formData.id === conn.id) {
      return false;
    }
    return !props.formData.jump_chain?.some((jumpHostId, index) => index !== currentIndex && jumpHostId === conn.id);
  });
};
</script>

<template>
  <div class="section-card">
    <h4 class="section-title">高级选项</h4>

    <div v-if="formData.type === 'SSH'" class="field-block">
      <label class="field-label">连接方式</label>
      <div class="segment-group">
        <button
          type="button"
          class="segment-btn"
          :class="{ active: advancedConnectionMode === 'proxy' }"
          @click="setConnectionMode('proxy')"
        >
          代理
        </button>
        <button
          type="button"
          class="segment-btn"
          :class="{ active: advancedConnectionMode === 'jump' }"
          @click="setConnectionMode('jump')"
        >
          跳板机
        </button>
      </div>
    </div>

    <div v-if="formData.type === 'SSH' && advancedConnectionMode === 'proxy'" class="field-block">
      <label for="conn-proxy" class="field-label">代理（可选）</label>
      <select id="conn-proxy" v-model="formData.proxy_id" class="field-select">
        <option :value="null">无代理</option>
        <option v-for="proxy in proxies" :key="proxy.id" :value="proxy.id">
          {{ proxy.name }} ({{ proxy.proxy_type }} - {{ proxy.host }}:{{ proxy.port }})
        </option>
      </select>
      <div v-if="isProxyLoading" class="hint-text">代理加载中...</div>
      <div v-if="proxyStoreError" class="error-text">{{ proxyStoreError }}</div>
    </div>

    <div v-if="formData.type === 'SSH' && advancedConnectionMode === 'jump'" class="field-block jump-chain-area">
      <label class="field-label">跳板机链配置</label>

      <div v-if="!formData.jump_chain || formData.jump_chain.length === 0" class="hint-text">
        还未添加跳板机。
      </div>

      <template v-if="formData.jump_chain">
        <div
          v-for="(jumpHostId, index) in formData.jump_chain"
          :key="index"
          class="jump-item"
        >
          <span class="jump-label">跳板机 {{ index + 1 }}</span>
          <select v-model="formData.jump_chain[index]" class="field-select jump-select">
            <option :value="null">请选择跳板机</option>
            <option
              v-for="host in getAvailableJumpHostsForIndex(index)"
              :key="host.id"
              :value="host.id"
            >
              {{ host.name }}
            </option>
          </select>
          <button
            type="button"
            class="remove-jump-btn"
            title="移除此跳板机"
            @click="removeJumpHost(index)"
          >
            <i class="fas fa-times"></i>
          </button>
        </div>
      </template>

      <button type="button" class="add-jump-btn" @click="addJumpHost()">
        <i class="fas fa-plus"></i>
        <span>添加跳板机</span>
      </button>

      <div
        v-if="connections.filter(conn => String(conn.type).toUpperCase() === 'SSH' && (!isEditMode || conn.id !== formData.id)).length === 0"
        class="warning-text"
      >
        没有可用的 SSH 连接可作为跳板机，请先创建 SSH 连接。
      </div>
    </div>

    <div v-if="formData.type === 'RDP'" class="field-block option-group">
      <label class="field-label">RDP 连接选项</label>
      <div class="option-grid option-grid-2">
        <div class="field-block option-inline">
          <label class="field-label">宽度</label>
          <input v-model.number="formData.rdpOptions.width" type="number" min="0" class="field-input" placeholder="默认" />
        </div>
        <div class="field-block option-inline">
          <label class="field-label">高度</label>
          <input v-model.number="formData.rdpOptions.height" type="number" min="0" class="field-input" placeholder="默认" />
        </div>
      </div>

      <div class="option-grid option-grid-3">
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.fullscreen" type="checkbox" /> 全屏模式</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.multimon" type="checkbox" /> 多显示器</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.admin" type="checkbox" /> 管理员模式</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.restrictedAdmin" type="checkbox" /> 受限管理员</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.remoteGuard" type="checkbox" /> Remote Guard</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.compression" type="checkbox" /> 压缩</label>
      </div>

      <div class="option-grid option-grid-3">
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.drives" type="checkbox" /> 驱动器重定向</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.printers" type="checkbox" /> 打印机重定向</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.clipboard" type="checkbox" /> 剪贴板重定向</label>
      </div>

      <div class="option-grid option-grid-2">
        <div class="field-block option-inline">
          <label class="field-label">音频</label>
          <select v-model="formData.rdpOptions.audio" class="field-select">
            <option value="local">本地播放</option>
            <option value="remote">远端播放</option>
            <option value="none">禁用音频</option>
          </select>
        </div>
        <div class="field-block option-inline">
          <label class="field-label">色深</label>
          <select v-model.number="formData.rdpOptions.colorDepth" class="field-select">
            <option :value="15">15-bit</option>
            <option :value="16">16-bit</option>
            <option :value="24">24-bit</option>
            <option :value="32">32-bit</option>
          </select>
        </div>
      </div>

      <div class="field-block option-group-child">
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.rdpOptions.gateway.enabled" type="checkbox" /> 启用网关</label>
        <div v-if="formData.rdpOptions.gateway.enabled" class="option-grid option-grid-2">
          <div class="field-block option-inline">
            <label class="field-label">网关地址</label>
            <input v-model="formData.rdpOptions.gateway.host" type="text" class="field-input" placeholder="gateway.example.com" />
          </div>
          <div class="field-block option-inline">
            <label class="field-label">网关端口</label>
            <input v-model.number="formData.rdpOptions.gateway.port" type="number" min="1" max="65535" class="field-input" placeholder="443" />
          </div>
          <div class="field-block option-inline">
            <label class="field-label">网关用户名</label>
            <input v-model="formData.rdpOptions.gateway.username" type="text" class="field-input" />
          </div>
          <div class="field-block option-inline">
            <label class="field-label">网关密码</label>
            <input v-model="formData.rdpOptions.gateway.password" type="password" class="field-input" autocomplete="new-password" />
          </div>
        </div>
      </div>
    </div>

    <div v-if="formData.type === 'VNC'" class="field-block option-group">
      <label class="field-label">VNC 显示选项</label>
      <div class="option-grid option-grid-3">
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.vncOptions.viewOnly" type="checkbox" /> 只读模式</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.vncOptions.localCursor" type="checkbox" /> 本地光标</label>
        <label class="checkbox-row"><input class="checkbox-input" v-model="formData.vncOptions.sharedConnection" type="checkbox" /> 共享连接</label>
      </div>

      <div class="option-grid option-grid-2">
        <div class="field-block option-inline">
          <label class="field-label">图像质量 (0-9)</label>
          <input v-model.number="formData.vncOptions.quality" type="number" min="0" max="9" class="field-input" />
        </div>
        <div class="field-block option-inline">
          <label class="field-label">压缩级别 (0-9)</label>
          <input v-model.number="formData.vncOptions.compression" type="number" min="0" max="9" class="field-input" />
        </div>
      </div>
    </div>

    <div class="field-block">
      <label class="field-label">标签（可选）</label>
      <TagInput
        v-model="formData.tag_ids"
        :available-tags="tags"
        :allow-create="true"
        :allow-delete="true"
        placeholder="添加或选择标签..."
        @create-tag="emit('create-tag', $event)"
        @delete-tag="emit('delete-tag', $event)"
      />
      <div v-if="isTagLoading" class="hint-text">标签加载中...</div>
      <div v-if="tagStoreError" class="error-text">{{ tagStoreError }}</div>
    </div>

    <div class="field-block">
      <label for="conn-notes" class="field-label">备注</label>
      <textarea
        id="conn-notes"
        v-model="formData.notes"
        rows="3"
        class="field-textarea"
        placeholder="输入连接备注..."
      ></textarea>
    </div>
  </div>
</template>

<style scoped>
.section-card {
  display: flex;
  flex-direction: column;
  gap: 14px;
  padding: 14px;
  border: 1px solid var(--border);
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface1) 45%, transparent);
}

.section-title {
  margin: 0;
  padding-bottom: 8px;
  border-bottom: 1px solid color-mix(in srgb, var(--border) 70%, transparent);
  font-size: 16px;
  font-weight: 600;
  color: var(--text);
}

.field-block {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  font-size: 13px;
  font-weight: 500;
  color: var(--text-sub);
}

.segment-group {
  display: grid;
  grid-template-columns: repeat(2, minmax(0, 1fr));
  border: 1px solid var(--border);
  border-radius: 6px;
  overflow: hidden;
}

.segment-btn {
  height: 34px;
  border: none;
  border-right: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  font-size: 13px;
  cursor: pointer;
}

.segment-btn:last-child {
  border-right: none;
}

.segment-btn.active {
  background: var(--blue);
  color: #ffffff;
}

.field-input,
.field-select,
.field-textarea {
  width: 100%;
  padding: 8px 12px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: var(--bg-base);
  color: var(--text);
  font-size: 13px;
}

.field-select {
  appearance: none;
  background-image: url("data:image/svg+xml,%3csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 16 16'%3e%3cpath fill='none' stroke='%238e98a0' stroke-linecap='round' stroke-linejoin='round' stroke-width='2' d='M2 5l6 6 6-6'/%3e%3c/svg%3e");
  background-repeat: no-repeat;
  background-size: 14px 10px;
  background-position: right 10px center;
  padding-right: 30px;
}

.field-input:focus,
.field-select:focus,
.field-textarea:focus {
  outline: none;
  border-color: var(--blue);
  box-shadow: 0 0 0 1px var(--blue);
}

.field-textarea {
  resize: vertical;
}

.jump-chain-area {
  gap: 10px;
}

.jump-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: color-mix(in srgb, var(--bg-base) 65%, transparent);
}

.jump-label {
  font-size: 12px;
  color: var(--text-sub);
  white-space: nowrap;
}

.jump-select {
  flex: 1;
  min-width: 0;
}

.remove-jump-btn {
  width: 28px;
  height: 28px;
  border: 1px solid var(--red);
  border-radius: 4px;
  background: transparent;
  color: var(--red);
  cursor: pointer;
}

.remove-jump-btn:hover {
  background: color-mix(in srgb, var(--red) 15%, transparent);
}

.add-jump-btn {
  width: 100%;
  height: 34px;
  border: 1px dashed var(--blue);
  border-radius: 6px;
  background: transparent;
  color: var(--blue);
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  cursor: pointer;
}

.add-jump-btn:hover {
  background: color-mix(in srgb, var(--blue) 12%, transparent);
}

.hint-text {
  font-size: 12px;
  color: var(--text-sub);
}

.error-text {
  font-size: 12px;
  color: var(--red);
}

.option-group {
  gap: 12px;
}

.option-group-child {
  gap: 10px;
  padding: 10px;
  border: 1px solid var(--border);
  border-radius: 6px;
  background: color-mix(in srgb, var(--bg-base) 72%, transparent);
}

.option-grid {
  display: grid;
  gap: 10px;
}

.option-grid-2 {
  grid-template-columns: repeat(2, minmax(0, 1fr));
}

.option-grid-3 {
  grid-template-columns: repeat(3, minmax(0, 1fr));
}

.option-inline {
  gap: 6px;
}

.checkbox-row {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 22px;
  font-size: 13px;
  color: var(--text-sub);
}

.checkbox-input {
  width: 14px;
  height: 14px;
  margin: 0;
  accent-color: var(--blue);
}


@media (max-width: 920px) {
  .option-grid-2,
  .option-grid-3 {
    grid-template-columns: 1fr;
  }
}

.warning-text {
  padding: 8px 10px;
  border-radius: 6px;
  background: color-mix(in srgb, var(--yellow) 20%, transparent);
  color: var(--text);
  font-size: 12px;
}
</style>

<template>
  <div class="ai-settings-panel">
    <div class="ai-tab-bar">
      <button
        v-for="tab in tabs"
        :key="tab.key"
        class="tab-btn"
        :class="{ active: activeTab === tab.key }"
        type="button"
        @click="activeTab = tab.key"
      >
        {{ tab.label }}
      </button>
    </div>

    <p v-if="aiStore.error" class="banner error">{{ aiStore.error }}</p>

    <section v-if="activeTab === 'channels'" class="panel-card">
      <div class="section-title-row">
        <h3 class="section-title">AI 渠道</h3>
        <button class="btn btn-primary" type="button" @click="openAddChannel">添加渠道</button>
      </div>

      <div v-if="channels.length === 0" class="empty-state">暂无 AI 渠道，请先添加。</div>

      <div v-else class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>名称</th>
              <th>类型</th>
              <th>API 地址</th>
              <th>启用</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="channel in channels" :key="channel.id">
              <td>{{ channel.name }}</td>
              <td>{{ getProviderName(channel.type) }}</td>
              <td class="endpoint-cell">{{ channel.apiEndpoint || getDefaultEndpoint(channel.type) }}</td>
              <td>
                <input
                  class="switch-input"
                  type="checkbox"
                  :checked="channel.enabled"
                  @change="onChannelToggleChange(channel, $event)"
                >
              </td>
              <td class="actions-cell">
                <button class="text-btn" type="button" :disabled="verifyingChannelId === channel.id" @click="verifyChannel(channel)">
                  {{ verifyingChannelId === channel.id ? '验证中...' : '验证' }}
                </button>
                <button class="text-btn" type="button" :disabled="fetchingChannelId === channel.id" @click="fetchModels(channel)">
                  {{ fetchingChannelId === channel.id ? '获取中...' : '获取模型' }}
                </button>
                <button class="text-btn" type="button" @click="editChannel(channel)">编辑</button>
                <button class="text-btn danger" type="button" @click="removeChannel(channel)">删除</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section v-if="activeTab === 'models'" class="panel-card">
      <div class="section-title-row">
        <div class="title-with-filter">
          <h3 class="section-title">AI 模型</h3>
          <AppSelect
            v-model="filterChannelId"
            class="channel-filter"
            variant="input"
            :options="channelFilterOptions"
            placeholder="筛选渠道"
          />
        </div>
        <button class="btn btn-primary" type="button" @click="showAddModelDialog = true">手动添加模型</button>
      </div>

      <div v-if="filteredModels.length === 0" class="empty-state">暂无模型，请先获取或添加模型。</div>

      <div v-else class="table-wrap">
        <table class="data-table">
          <thead>
            <tr>
              <th>模型名称</th>
              <th>模型 ID</th>
              <th>所属渠道</th>
              <th>上下文窗口</th>
              <th>类型</th>
              <th>默认</th>
              <th>操作</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="model in filteredModels" :key="model.id">
              <td>{{ model.displayName }}</td>
              <td class="mono">{{ model.modelId }}</td>
              <td>{{ aiStore.getChannelName(model.channelId) }}</td>
              <td>{{ formatContextWindow(model.contextWindow) }}</td>
              <td>{{ model.type === 'auto' ? '自动' : '手动' }}</td>
              <td>{{ aiStore.config.defaultModelId === model.id ? '✓' : '' }}</td>
              <td class="actions-cell">
                <button
                  v-if="aiStore.config.defaultModelId !== model.id"
                  class="text-btn"
                  type="button"
                  @click="setDefaultModel(model.id)"
                >
                  设为默认
                </button>
                <button class="text-btn danger" type="button" @click="removeModel(model.id, model.displayName)">删除</button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </section>

    <section v-if="activeTab === 'advanced'" class="panel-card">
      <h3 class="section-title">AI 请求参数</h3>
      <div class="advanced-params-wrap">
        <div class="param-row">
          <label class="param-label" for="ai-temperature">Temperature</label>
          <div class="param-main">
            <input
              id="ai-temperature"
              v-model.number="localConfig.temperature"
              class="range-input"
              type="range"
              min="0"
              max="2"
              step="0.1"
              :style="temperatureTrackStyle"
              @input="markChanged"
            >
            <div class="stepper-control stepper-control-sm">
              <button class="stepper-btn" type="button" @click="adjustTemperature(-0.1)">−</button>
              <input
                v-model.number="localConfig.temperature"
                class="stepper-value"
                type="number"
                min="0"
                max="2"
                step="0.1"
                inputmode="decimal"
                @input="markChanged"
                @blur="normalizeTemperature"
              >
              <button class="stepper-btn" type="button" @click="adjustTemperature(0.1)">+</button>
            </div>
          </div>
          <span class="param-hint">控制输出的随机性 (0.0 - 2.0)</span>
        </div>

        <div class="param-row">
          <label class="param-label" for="ai-max-tokens">Max Tokens</label>
          <div class="param-main">
            <div class="stepper-control">
              <button class="stepper-btn" type="button" @click="adjustMaxTokens(-100)">−</button>
              <input
                id="ai-max-tokens"
                v-model.number="localConfig.maxTokens"
                class="stepper-value"
                type="number"
                min="100"
                max="100000"
                step="100"
                inputmode="numeric"
                @input="markChanged"
                @blur="normalizeMaxTokens"
              >
              <button class="stepper-btn" type="button" @click="adjustMaxTokens(100)">+</button>
            </div>
          </div>
          <span class="param-hint">最大生成令牌数 (100 - 100000)</span>
        </div>

        <div class="param-row">
          <label class="param-label" for="ai-timeout">请求超时</label>
          <div class="param-main">
            <div class="stepper-control">
              <button class="stepper-btn" type="button" @click="adjustTimeout(-5000)">−</button>
              <input
                id="ai-timeout"
                v-model.number="localConfig.timeout"
                class="stepper-value"
                type="number"
                min="5000"
                max="120000"
                step="5000"
                inputmode="numeric"
                @input="markChanged"
                @blur="normalizeTimeout"
              >
              <button class="stepper-btn" type="button" @click="adjustTimeout(5000)">+</button>
            </div>
          </div>
          <span class="param-hint">毫秒(5000 - 120000)</span>
        </div>
      </div>

      <div class="actions">
        <button class="btn btn-primary" type="button" :disabled="!hasChanges || savingConfig" @click="saveConfig">
          {{ savingConfig ? '保存中...' : '保存参数' }}
        </button>
      </div>
    </section>

    <section v-if="activeTab === 'prompts'" class="panel-card">
      <h3 class="section-title">提示词模板</h3>
      <p class="section-desc">可使用 `{content}` 和 `{language}` 变量。</p>

      <div class="field-row">
        <label class="field-label">代码解释模板</label>
        <textarea v-model="localConfig.prompts.explain" class="form-control textarea" @input="markChanged"></textarea>
      </div>

      <div class="field-row">
        <label class="field-label">代码优化模板</label>
        <textarea v-model="localConfig.prompts.optimize" class="form-control textarea" @input="markChanged"></textarea>
      </div>

      <div class="field-row">
        <label class="field-label">代码生成模板</label>
        <textarea v-model="localConfig.prompts.write" class="form-control textarea" @input="markChanged"></textarea>
      </div>

      <div class="actions">
        <button class="btn btn-primary" type="button" :disabled="!hasChanges || savingConfig" @click="saveConfig">
          {{ savingConfig ? '保存中...' : '保存模板' }}
        </button>
      </div>
    </section>

    <AIChannelForm
      v-model:visible="showChannelDialog"
      :channel="editingChannel"
      @saved="reloadChannelsAndModels"
    />

    <AIModelForm
      v-model:visible="showAddModelDialog"
      :channels="channels"
      @saved="reloadModels"
    />
  </div>
</template>

<script setup lang="ts">
import { computed, onMounted, ref } from 'vue';
import AppSelect from '@/components/AppSelect.vue';
import AIChannelForm from '@/components/AI/AIChannelForm.vue';
import AIModelForm from '@/components/AI/AIModelForm.vue';
import { useAIStore } from '@/stores/ai';
import { useUiNotificationsStore } from '@/stores/uiNotifications';
import type { AIChannel, AIConfig, AIModel, AIProviderType } from '@/types/ai';

type TabKey = 'channels' | 'models' | 'advanced' | 'prompts';
const DEFAULT_PROMPT_EXPLAIN =
  '请作为一名资深开发人员，详细分析并解释以下代码片段的主要功能和目的。\n\n```{language}\n{content}\n```';
const DEFAULT_PROMPT_OPTIMIZE =
  'Optimize this code:\n\n```{language}\n{content}\n```\n\nReturn only the optimized code without explanations or markdown code blocks.';
const DEFAULT_PROMPT_WRITE =
  'Write code based on this description: {content}\n\nLanguage: {language}\n\nReturn only the code without explanations or markdown code blocks.';

const tabs: Array<{ key: TabKey; label: string }> = [
  { key: 'channels', label: '渠道管理' },
  { key: 'models', label: '模型管理' },
  { key: 'advanced', label: '高级设置' },
  { key: 'prompts', label: '提示词设置' },
];

const aiStore = useAIStore();
const notifications = useUiNotificationsStore();

const activeTab = ref<TabKey>('channels');
const showChannelDialog = ref(false);
const showAddModelDialog = ref(false);
const editingChannel = ref<AIChannel | null>(null);
const filterChannelId = ref('');
const verifyingChannelId = ref<string | null>(null);
const fetchingChannelId = ref<string | null>(null);
const savingConfig = ref(false);
const hasChanges = ref(false);

const channels = computed(() => aiStore.channels);
const models = computed(() => aiStore.models);

const filteredModels = computed(() => {
  if (!filterChannelId.value) {
    return models.value;
  }
  return models.value.filter((model) => model.channelId === filterChannelId.value);
});

const channelFilterOptions = computed(() => [
  { label: '全部渠道', value: '' },
  ...channels.value.map((channel) => ({ label: channel.name, value: channel.id })),
]);

const localConfig = ref<AIConfig>({
  defaultModelId: undefined,
  temperature: 0.7,
  maxTokens: 4000,
  timeout: 60000,
  prompts: {
    explain: DEFAULT_PROMPT_EXPLAIN,
    optimize: DEFAULT_PROMPT_OPTIMIZE,
    write: DEFAULT_PROMPT_WRITE,
  },
});

const syncLocalConfig = () => {
  localConfig.value = JSON.parse(JSON.stringify(aiStore.config)) as AIConfig;
  localConfig.value.temperature = Number(clampValue(localConfig.value.temperature, 0, 2).toFixed(1));
  localConfig.value.maxTokens = Math.round(clampValue(localConfig.value.maxTokens, 100, 100000));
  localConfig.value.timeout = Math.round(clampValue(localConfig.value.timeout, 5000, 120000));
  if (!localConfig.value.prompts.explain.trim()) {
    localConfig.value.prompts.explain = DEFAULT_PROMPT_EXPLAIN;
  }
  if (!localConfig.value.prompts.optimize.trim()) {
    localConfig.value.prompts.optimize = DEFAULT_PROMPT_OPTIMIZE;
  }
  if (!localConfig.value.prompts.write.trim()) {
    localConfig.value.prompts.write = DEFAULT_PROMPT_WRITE;
  }
  hasChanges.value = false;
};

const getProviderName = (type: AIProviderType) => {
  if (type === 'openai') return 'OpenAI';
  if (type === 'anthropic') return 'Anthropic';
  if (type === 'gemini') return 'Gemini';
  return 'OpenAI 兼容';
};

const getDefaultEndpoint = (type: AIProviderType) => {
  if (type === 'openai') return 'https://api.openai.com/v1';
  if (type === 'anthropic') return 'https://api.anthropic.com/v1';
  if (type === 'gemini') return 'https://generativelanguage.googleapis.com/v1beta';
  return '自定义';
};

const formatContextWindow = (value: number) => {
  if (value >= 1_000_000) return `${(value / 1_000_000).toFixed(1)}M`;
  if (value >= 1_000) return `${Math.round(value / 1_000)}K`;
  return String(value);
};

const markChanged = () => {
  hasChanges.value = true;
};

const clampValue = (value: number, min: number, max: number) => Math.min(max, Math.max(min, value));

const temperatureTrackStyle = computed(() => {
  const ratio = clampValue(localConfig.value.temperature, 0, 2) / 2;
  const progress = Math.round(ratio * 100);
  return {
    '--temperature-progress': `${progress}%`,
  };
});

const normalizeTemperature = () => {
  const raw = Number.isFinite(localConfig.value.temperature) ? localConfig.value.temperature : 0.7;
  const normalized = Number(clampValue(raw, 0, 2).toFixed(1));
  if (normalized !== localConfig.value.temperature) {
    localConfig.value.temperature = normalized;
    markChanged();
  }
};

const normalizeMaxTokens = () => {
  const raw = Number.isFinite(localConfig.value.maxTokens) ? localConfig.value.maxTokens : 4000;
  const normalized = Math.round(clampValue(raw, 100, 100000));
  if (normalized !== localConfig.value.maxTokens) {
    localConfig.value.maxTokens = normalized;
    markChanged();
  }
};

const normalizeTimeout = () => {
  const raw = Number.isFinite(localConfig.value.timeout) ? localConfig.value.timeout : 60000;
  const normalized = Math.round(clampValue(raw, 5000, 120000));
  if (normalized !== localConfig.value.timeout) {
    localConfig.value.timeout = normalized;
    markChanged();
  }
};

const adjustTemperature = (delta: number) => {
  const next = Number((localConfig.value.temperature + delta).toFixed(1));
  localConfig.value.temperature = clampValue(next, 0, 2);
  markChanged();
};

const adjustMaxTokens = (delta: number) => {
  localConfig.value.maxTokens = Math.round(clampValue(localConfig.value.maxTokens + delta, 100, 100000));
  markChanged();
};

const adjustTimeout = (delta: number) => {
  localConfig.value.timeout = Math.round(clampValue(localConfig.value.timeout + delta, 5000, 120000));
  markChanged();
};

const reloadChannelsAndModels = async () => {
  await Promise.all([aiStore.loadChannels(), aiStore.loadModels()]);
};

const reloadModels = async () => {
  await aiStore.loadModels();
};

const openAddChannel = () => {
  editingChannel.value = null;
  showChannelDialog.value = true;
};

const editChannel = (channel: AIChannel) => {
  editingChannel.value = { ...channel };
  showChannelDialog.value = true;
};

const verifyChannel = async (channel: AIChannel) => {
  verifyingChannelId.value = channel.id;
  try {
    const ok = await aiStore.verifyChannel(channel.id);
    notifications.addNotification(ok ? 'success' : 'error', ok ? '渠道验证成功' : '渠道验证失败');
  } finally {
    verifyingChannelId.value = null;
  }
};

const fetchModels = async (channel: AIChannel) => {
  fetchingChannelId.value = channel.id;
  try {
    const list = await aiStore.fetchModels(channel.id);
    notifications.addNotification('success', `已获取 ${list.length} 个模型`);
  } catch (error) {
    notifications.addNotification('error', error instanceof Error ? error.message : '获取模型失败');
  } finally {
    fetchingChannelId.value = null;
  }
};

const toggleChannelEnabled = async (channel: AIChannel, enabled: boolean) => {
  const previous = channel.enabled;
  channel.enabled = enabled;
  try {
    await aiStore.updateChannel(channel.id, { enabled });
    notifications.addNotification('success', enabled ? '渠道已启用' : '渠道已禁用');
  } catch (error) {
    channel.enabled = previous;
    notifications.addNotification('error', error instanceof Error ? error.message : '更新渠道状态失败');
  }
};

const onChannelToggleChange = (channel: AIChannel, event: Event) => {
  const target = event.target as HTMLInputElement | null;
  const enabled = target?.checked ?? false;
  void toggleChannelEnabled(channel, enabled);
};

const removeChannel = async (channel: AIChannel) => {
  const confirmed = window.confirm(`确定删除渠道“${channel.name}”？其模型也会被删除。`);
  if (!confirmed) {
    return;
  }
  try {
    await aiStore.deleteChannel(channel.id);
    notifications.addNotification('success', '渠道已删除');
  } catch (error) {
    notifications.addNotification('error', error instanceof Error ? error.message : '删除渠道失败');
  }
};

const setDefaultModel = async (modelId: string) => {
  try {
    await aiStore.setDefaultModel(modelId);
    localConfig.value.defaultModelId = modelId;
    notifications.addNotification('success', '默认模型设置成功');
  } catch (error) {
    notifications.addNotification('error', error instanceof Error ? error.message : '设置默认模型失败');
  }
};

const removeModel = async (modelId: string, displayName: string) => {
  const confirmed = window.confirm(`确定删除模型“${displayName}”？`);
  if (!confirmed) {
    return;
  }
  try {
    await aiStore.deleteModel(modelId);
    notifications.addNotification('success', '模型已删除');
  } catch (error) {
    notifications.addNotification('error', error instanceof Error ? error.message : '删除模型失败');
  }
};

const saveConfig = async () => {
  savingConfig.value = true;
  try {
    await aiStore.updateConfig(localConfig.value);
    hasChanges.value = false;
    notifications.addNotification('success', 'AI 配置已保存');
  } catch (error) {
    notifications.addNotification('error', error instanceof Error ? error.message : '保存配置失败');
  } finally {
    savingConfig.value = false;
  }
};

onMounted(async () => {
  await aiStore.loadAll();
  syncLocalConfig();
});
</script>

<style scoped>
.ai-settings-panel {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.ai-tab-bar {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.tab-btn {
  height: 32px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0 12px;
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
  background: var(--bg-mantle);
  color: var(--text-sub);
}

.tab-btn:hover {
  color: var(--text);
  background: var(--bg-surface1);
}

.tab-btn.active {
  color: var(--button-text-color);
  border-color: var(--button-bg-color);
  background: var(--button-bg-color);
}

.banner {
  margin: 0;
  border-radius: 8px;
  border: 1px solid var(--border);
  background: var(--bg-surface0);
  color: var(--text-sub);
  padding: 8px 10px;
  font-size: calc(12px + var(--ui-font-size-offset));
}

.banner.error {
  border-color: color-mix(in srgb, var(--red) 40%, var(--border));
  color: var(--red);
}

.panel-card {
  border: 1px solid var(--border);
  border-radius: 10px;
  background: var(--bg-surface0);
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.section-title-row {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 10px;
}

.title-with-filter {
  display: flex;
  align-items: center;
  gap: 10px;
}

.section-title {
  margin: 0;
  font-size: calc(15px + var(--ui-font-size-offset));
  color: var(--text);
}

.section-desc {
  margin: -6px 0 0;
  color: var(--text-sub);
  font-size: calc(12px + var(--ui-font-size-offset));
}

.channel-filter {
  width: 220px;
}

.table-wrap {
  overflow: auto;
  border: 1px solid var(--border);
  border-radius: 8px;
}

.data-table {
  width: 100%;
  border-collapse: collapse;
  font-size: calc(13px + var(--ui-font-size-offset));
  min-width: 840px;
}

.data-table th,
.data-table td {
  border-bottom: 1px solid var(--border);
  padding: 8px 10px;
  text-align: left;
  vertical-align: middle;
}

.data-table th {
  background: var(--bg-mantle);
  color: var(--text-sub);
  font-weight: 600;
}

.data-table tr:last-child td {
  border-bottom: none;
}

.endpoint-cell {
  max-width: 300px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.actions-cell {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
}

.text-btn {
  border: none;
  background: transparent;
  color: var(--blue);
  cursor: pointer;
  font-size: calc(12px + var(--ui-font-size-offset));
}

.text-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.text-btn.danger {
  color: var(--red);
}

.empty-state {
  border: 1px dashed var(--border);
  border-radius: 8px;
  color: var(--text-sub);
  text-align: center;
  padding: 16px;
  font-size: calc(13px + var(--ui-font-size-offset));
}

.field-row {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.field-label {
  color: var(--text-sub);
  font-size: calc(13px + var(--ui-font-size-offset));
}

.advanced-params-wrap {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.param-row {
  display: flex;
  align-items: center;
  gap: 14px;
  min-height: 38px;
}

.param-label {
  width: 140px;
  flex: 0 0 140px;
  color: var(--text-sub);
  font-size: calc(14px + var(--ui-font-size-offset));
}

.param-main {
  display: flex;
  align-items: center;
  gap: 10px;
  min-width: 330px;
}

.param-hint {
  color: var(--text-sub);
  font-size: calc(14px + var(--ui-font-size-offset));
  white-space: nowrap;
}

.range-input {
  width: 140px;
  height: 6px;
  border-radius: 999px;
  outline: none;
  border: none;
  appearance: none;
  background: linear-gradient(
    to right,
    var(--blue) 0%,
    var(--blue) var(--temperature-progress, 35%),
    color-mix(in srgb, var(--border) 72%, var(--bg-surface1)) var(--temperature-progress, 35%),
    color-mix(in srgb, var(--border) 72%, var(--bg-surface1)) 100%
  );
}

.range-input::-webkit-slider-thumb {
  appearance: none;
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: var(--bg-surface0);
  border: 2px solid var(--blue);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--blue) 25%, transparent);
  cursor: pointer;
}

.range-input::-moz-range-thumb {
  width: 16px;
  height: 16px;
  border-radius: 999px;
  background: var(--bg-surface0);
  border: 2px solid var(--blue);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--blue) 25%, transparent);
  cursor: pointer;
}

.range-input::-moz-range-track {
  height: 6px;
  border-radius: 999px;
  border: none;
  background: linear-gradient(
    to right,
    var(--blue) 0%,
    var(--blue) var(--temperature-progress, 35%),
    color-mix(in srgb, var(--border) 72%, var(--bg-surface1)) var(--temperature-progress, 35%),
    color-mix(in srgb, var(--border) 72%, var(--bg-surface1)) 100%
  );
}

.stepper-control {
  display: flex;
  align-items: center;
  overflow: hidden;
  min-width: 206px;
  height: 32px;
  border: 1px solid color-mix(in srgb, var(--border) 82%, var(--blue));
  border-radius: 8px;
  background: color-mix(in srgb, var(--bg-surface1) 86%, var(--bg-base));
  transition: border-color 0.18s ease, box-shadow 0.18s ease;
}

.stepper-control-sm {
  min-width: 160px;
}

.stepper-btn {
  flex: 0 0 34px;
  width: 34px;
  height: 100%;
  border: none;
  border-right: 1px solid color-mix(in srgb, var(--border) 82%, var(--blue));
  background: color-mix(in srgb, var(--bg-surface1) 72%, var(--bg-base));
  color: var(--text-sub);
  font-size: calc(17px + var(--ui-font-size-offset));
  line-height: 1;
  cursor: pointer;
  transition: background 0.18s ease, color 0.18s ease, border-color 0.18s ease;
}

.stepper-btn:last-child {
  border-right: none;
  border-left: 1px solid color-mix(in srgb, var(--border) 82%, var(--blue));
}

.stepper-btn:hover {
  background: color-mix(in srgb, var(--blue) 16%, var(--bg-surface1));
  color: var(--text);
}

.stepper-control:focus-within {
  border-color: var(--blue);
  box-shadow: 0 0 0 2px color-mix(in srgb, var(--blue) 24%, transparent);
}

.stepper-value {
  flex: 1;
  min-width: 0;
  width: 100%;
  height: 100%;
  border: none;
  outline: none;
  text-align: center;
  color: var(--text);
  font-size: calc(17px + var(--ui-font-size-offset));
  font-weight: 600;
  background: color-mix(in srgb, var(--bg-surface1) 82%, var(--bg-base));
  padding: 0 6px;
}

.stepper-value::-webkit-outer-spin-button,
.stepper-value::-webkit-inner-spin-button {
  appearance: none;
  margin: 0;
}

.stepper-value[type='number'] {
  appearance: textfield;
  -moz-appearance: textfield;
}

.stepper-value:focus {
  background: color-mix(in srgb, var(--blue) 8%, var(--bg-surface1));
}

.stepper-control-sm .stepper-value {
  font-size: calc(15px + var(--ui-font-size-offset));
}

.mono {
  font-family: 'Cascadia Mono', Consolas, 'Courier New', monospace;
}

.form-control {
  width: 100%;
  min-height: 34px;
  border-radius: 6px;
  border: 1px solid var(--border);
  background: var(--bg-base);
  color: var(--text);
  padding: 6px 10px;
  font-size: calc(13px + var(--ui-font-size-offset));
}

.form-control:focus {
  outline: none;
  border-color: var(--blue);
}

.textarea {
  min-height: 100px;
  resize: vertical;
}

.switch-input {
  width: 14px;
  height: 14px;
  accent-color: var(--blue);
}

.actions {
  display: flex;
  justify-content: flex-end;
}

.btn {
  height: 32px;
  border: 1px solid var(--border);
  border-radius: 6px;
  padding: 0 12px;
  font-size: calc(13px + var(--ui-font-size-offset));
  cursor: pointer;
  background: transparent;
  color: var(--text);
}

.btn:hover {
  background: var(--bg-surface1);
}

.btn-primary {
  border-color: var(--button-bg-color);
  background: var(--button-bg-color);
  color: var(--button-text-color);
}

.btn-primary:hover {
  background: var(--button-hover-bg-color);
}

.btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.number-input {
  width: 168px;
  min-width: 140px;
}

.with-unit {
  text-align: right;
  padding-right: 12px;
}

.unit-badge {
  flex-shrink: 0;
  height: 24px;
  min-width: 56px;
  border: 1px solid color-mix(in srgb, var(--border) 72%, var(--blue));
  border-radius: 999px;
  padding: 0 10px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: calc(11px + var(--ui-font-size-offset));
  color: var(--text-sub);
  background: color-mix(in srgb, var(--bg-mantle) 88%, black);
}

.note-block {
  border: 1px solid var(--border);
  border-radius: 8px;
  background: var(--bg-mantle);
  color: var(--text-sub);
  font-size: calc(12px + var(--ui-font-size-offset));
  padding: 10px 12px;
}

.note-block p {
  margin: 0 0 6px;
}

.note-block ul {
  margin: 0;
  padding-left: 16px;
  display: grid;
  gap: 4px;
}

.stepper-value::placeholder {
  color: var(--text-sub);
}

.stepper-control .stepper-btn:focus-visible,
.stepper-value:focus-visible,
.range-input:focus-visible {
  outline: 2px solid color-mix(in srgb, var(--blue) 45%, transparent);
  outline-offset: 1px;
}
@media (max-width: 980px) {
  .title-with-filter {
    flex-direction: column;
    align-items: flex-start;
  }

  .channel-filter {
    width: 100%;
  }

  .param-row {
    flex-wrap: wrap;
    gap: 8px;
  }

  .param-label {
    width: 100%;
    flex-basis: 100%;
  }

  .param-main {
    width: 100%;
    min-width: 0;
  }

  .param-hint {
    white-space: normal;
    font-size: calc(13px + var(--ui-font-size-offset));
  }
}
</style>

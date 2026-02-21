import { computed, ref } from 'vue';
import { defineStore } from 'pinia';
import { aiApi } from '@/lib/api-ai';
import type { AIAction, AIChannel, AIChatMessage, AIConfig, AIModel } from '@/types/ai';

const DEFAULT_PROMPT_EXPLAIN =
  '请作为一名资深开发人员，详细分析并解释以下代码片段的主要功能和目的。\n\n```{language}\n{content}\n```';
const DEFAULT_PROMPT_OPTIMIZE =
  'Optimize this code:\n\n```{language}\n{content}\n```\n\nReturn only the optimized code without explanations or markdown code blocks.';
const DEFAULT_PROMPT_WRITE =
  'Write code based on this description: {content}\n\nLanguage: {language}\n\nReturn only the code without explanations or markdown code blocks.';

const defaultConfig = (): AIConfig => ({
  defaultModelId: undefined,
  temperature: 0.7,
  maxTokens: 2000,
  timeout: 30000,
  prompts: {
    explain: DEFAULT_PROMPT_EXPLAIN,
    optimize: DEFAULT_PROMPT_OPTIMIZE,
    write: DEFAULT_PROMPT_WRITE,
  },
});

export const useAIStore = defineStore('ai', () => {
  const channels = ref<AIChannel[]>([]);
  const models = ref<AIModel[]>([]);
  const config = ref<AIConfig>(defaultConfig());
  const messages = ref<AIChatMessage[]>([]);

  const loading = ref(false);
  const chatLoading = ref(false);
  const error = ref<string | null>(null);

  const defaultModel = computed(() => {
    if (!config.value.defaultModelId) {
      return null;
    }
    return models.value.find((item) => item.id === config.value.defaultModelId) ?? null;
  });

  const enabledChannels = computed(() => channels.value.filter((channel) => channel.enabled));

  const hasDefaultModel = computed(() => !!defaultModel.value);

  const modelsByChannel = computed(() => {
    const grouped: Record<string, AIModel[]> = {};
    for (const model of models.value) {
      if (!grouped[model.channelId]) {
        grouped[model.channelId] = [];
      }
      grouped[model.channelId].push(model);
    }
    return grouped;
  });

  const getChannelName = (channelId: string): string => {
    return channels.value.find((channel) => channel.id === channelId)?.name ?? '未知渠道';
  };

  const setError = (value: unknown, fallback: string) => {
    if (value instanceof Error && value.message.trim()) {
      error.value = value.message;
      return;
    }
    if (typeof value === 'string' && value.trim()) {
      error.value = value;
      return;
    }
    error.value = fallback;
  };

  async function loadChannels() {
    loading.value = true;
    error.value = null;
    try {
      channels.value = await aiApi.getAllChannels();
    } catch (err) {
      setError(err, '加载 AI 渠道失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function loadModels() {
    loading.value = true;
    error.value = null;
    try {
      models.value = await aiApi.getAllModels();
    } catch (err) {
      setError(err, '加载 AI 模型失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function loadConfig() {
    error.value = null;
    try {
      config.value = await aiApi.getConfig();
    } catch (err) {
      setError(err, '加载 AI 配置失败');
      throw err;
    }
  }

  async function loadAll() {
    await Promise.all([loadChannels(), loadModels(), loadConfig()]);
  }

  async function addChannel(data: Omit<AIChannel, 'id' | 'createdAt' | 'updatedAt'>) {
    loading.value = true;
    error.value = null;
    try {
      const channel = await aiApi.addChannel({
        name: data.name,
        type: data.type,
        apiKey: data.apiKey,
        apiEndpoint: data.apiEndpoint ?? undefined,
        enabled: data.enabled,
      });
      channels.value.push(channel);
      return channel;
    } catch (err) {
      setError(err, '添加 AI 渠道失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function updateChannel(id: string, updates: Partial<AIChannel>) {
    loading.value = true;
    error.value = null;
    try {
      await aiApi.updateChannel(id, {
        name: updates.name,
        type: updates.type,
        apiKey: updates.apiKey,
        apiEndpoint: updates.apiEndpoint ?? undefined,
        enabled: updates.enabled,
      });
      const index = channels.value.findIndex((item) => item.id === id);
      if (index >= 0) {
        channels.value[index] = {
          ...channels.value[index],
          ...updates,
          updatedAt: Date.now(),
        };
      }
    } catch (err) {
      setError(err, '更新 AI 渠道失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function deleteChannel(id: string) {
    loading.value = true;
    error.value = null;
    try {
      await aiApi.deleteChannel(id);
      channels.value = channels.value.filter((item) => item.id !== id);
      models.value = models.value.filter((item) => item.channelId !== id);
      if (config.value.defaultModelId && !models.value.some((item) => item.id === config.value.defaultModelId)) {
        config.value.defaultModelId = undefined;
      }
    } catch (err) {
      setError(err, '删除 AI 渠道失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function verifyChannel(id: string): Promise<boolean> {
    loading.value = true;
    error.value = null;
    try {
      return await aiApi.verifyChannel(id);
    } catch (err) {
      setError(err, '验证 AI 渠道失败');
      return false;
    } finally {
      loading.value = false;
    }
  }

  async function fetchModels(channelId: string): Promise<AIModel[]> {
    loading.value = true;
    error.value = null;
    try {
      const fetched = await aiApi.fetchModels(channelId);
      const keep = models.value.filter((item) => !(item.channelId === channelId && item.type === 'auto'));
      models.value = [...keep, ...fetched];
      return fetched;
    } catch (err) {
      setError(err, '获取 AI 模型失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function addModel(data: Omit<AIModel, 'id' | 'createdAt'>) {
    loading.value = true;
    error.value = null;
    try {
      const model = await aiApi.addModel({
        modelId: data.modelId,
        displayName: data.displayName,
        channelId: data.channelId,
        contextWindow: data.contextWindow,
        type: data.type,
      });
      models.value.push(model);
      return model;
    } catch (err) {
      setError(err, '添加 AI 模型失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function deleteModel(id: string) {
    loading.value = true;
    error.value = null;
    try {
      await aiApi.deleteModel(id);
      models.value = models.value.filter((item) => item.id !== id);
      if (config.value.defaultModelId === id) {
        config.value.defaultModelId = undefined;
      }
    } catch (err) {
      setError(err, '删除 AI 模型失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function setDefaultModel(modelId: string) {
    loading.value = true;
    error.value = null;
    try {
      await aiApi.setDefaultModel(modelId);
      config.value.defaultModelId = modelId;
    } catch (err) {
      setError(err, '设置默认模型失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function updateConfig(updates: Partial<AIConfig>) {
    loading.value = true;
    error.value = null;
    try {
      await aiApi.updateConfig({
        defaultModelId: updates.defaultModelId,
        temperature: updates.temperature,
        maxTokens: updates.maxTokens,
        timeout: updates.timeout,
        prompts: updates.prompts,
      });
      config.value = {
        ...config.value,
        ...updates,
        prompts: {
          ...config.value.prompts,
          ...(updates.prompts ?? {}),
        },
      };
    } catch (err) {
      setError(err, '更新 AI 配置失败');
      throw err;
    } finally {
      loading.value = false;
    }
  }

  async function sendRequest(action: AIAction, content: string, language?: string): Promise<string> {
    error.value = null;
    try {
      return await aiApi.request(action, content, language);
    } catch (err) {
      setError(err, 'AI 请求失败');
      throw err;
    }
  }

  async function sendRequestWithModel(
    action: AIAction,
    content: string,
    modelId: string,
    language?: string,
  ): Promise<string> {
    error.value = null;
    try {
      return await aiApi.requestWithModel(action, content, modelId, language);
    } catch (err) {
      setError(err, 'AI 请求失败');
      throw err;
    }
  }

  async function cancelRequest(requestId: string) {
    try {
      await aiApi.cancelRequest(requestId);
    } catch {
      // command is best-effort; ignore on frontend side
    }
  }

  async function loadChatHistory() {
    try {
      messages.value = await aiApi.getChatHistory();
    } catch {
      messages.value = [];
    }
  }

  async function saveChatHistory(nextMessages: AIChatMessage[] = messages.value) {
    try {
      await aiApi.saveChatHistory(nextMessages);
    } catch {
      // ignore persistence failures to avoid interrupting chat flow
    }
  }

  async function clearMessages() {
    messages.value = [];
    try {
      await aiApi.clearChatHistory();
    } catch {
      // ignore persistence failures
    }
  }

  async function getTerminalChatHistory(connectionId: string): Promise<AIChatMessage[]> {
    try {
      return await aiApi.getTerminalChatHistory(connectionId);
    } catch {
      return [];
    }
  }

  async function saveTerminalChatHistory(connectionId: string, nextMessages: AIChatMessage[]) {
    try {
      await aiApi.saveTerminalChatHistory(connectionId, nextMessages);
    } catch {
      // ignore persistence failures
    }
  }

  async function clearTerminalChatHistory(connectionId: string) {
    try {
      await aiApi.clearTerminalChatHistory(connectionId);
    } catch {
      // ignore persistence failures
    }
  }

  function clearError() {
    error.value = null;
  }

  return {
    channels,
    models,
    config,
    messages,
    loading,
    chatLoading,
    error,

    defaultModel,
    enabledChannels,
    hasDefaultModel,
    modelsByChannel,
    getChannelName,

    loadChannels,
    loadModels,
    loadConfig,
    loadAll,
    addChannel,
    updateChannel,
    deleteChannel,
    verifyChannel,
    fetchModels,
    addModel,
    deleteModel,
    setDefaultModel,
    updateConfig,
    sendRequest,
    sendRequestWithModel,
    cancelRequest,
    loadChatHistory,
    saveChatHistory,
    clearMessages,
    getTerminalChatHistory,
    saveTerminalChatHistory,
    clearTerminalChatHistory,
    clearError,
  };
});

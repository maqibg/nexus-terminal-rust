import { tauriInvoke } from './invoke';
import type {
  AIAction,
  AIChannel,
  AIChatMessage,
  AIConfig,
  AIModel,
  AIProviderType,
} from '@/types/ai';

interface AddChannelRequest {
  name: string;
  type: AIProviderType;
  apiKey: string;
  apiEndpoint?: string;
  enabled: boolean;
}

interface UpdateChannelRequest {
  name?: string;
  type?: AIProviderType;
  apiKey?: string;
  apiEndpoint?: string;
  enabled?: boolean;
}

interface AddModelRequest {
  modelId: string;
  displayName: string;
  channelId: string;
  contextWindow: number;
  type: 'auto' | 'manual';
}

interface AIConfigUpdate {
  defaultModelId?: string;
  temperature?: number;
  maxTokens?: number;
  timeout?: number;
  prompts?: AIConfig['prompts'];
}

export const aiApi = {
  getAllChannels: () => tauriInvoke<AIChannel[]>('ai_get_all_channels'),
  addChannel: (req: AddChannelRequest) => tauriInvoke<AIChannel>('ai_add_channel', { req }),
  updateChannel: (id: string, updates: UpdateChannelRequest) =>
    tauriInvoke<void>('ai_update_channel', { id, updates }),
  deleteChannel: (id: string) => tauriInvoke<void>('ai_delete_channel', { id }),
  verifyChannel: (id: string) => tauriInvoke<boolean>('ai_verify_channel', { id }),

  fetchModels: (channelId: string) =>
    tauriInvoke<AIModel[]>('ai_fetch_models', { channelId }),
  addModel: (req: AddModelRequest) => tauriInvoke<AIModel>('ai_add_model', { req }),
  deleteModel: (id: string) => tauriInvoke<void>('ai_delete_model', { id }),
  getAllModels: () => tauriInvoke<AIModel[]>('ai_get_all_models'),
  setDefaultModel: (modelId: string) => tauriInvoke<void>('ai_set_default_model', { modelId }),

  getConfig: () => tauriInvoke<AIConfig>('ai_get_config'),
  updateConfig: (updates: AIConfigUpdate) => tauriInvoke<void>('ai_update_config', { updates }),

  request: (action: AIAction, content: string, language?: string) =>
    tauriInvoke<string>('ai_request', { action, content, language }),
  requestWithModel: (action: AIAction, content: string, modelId: string, language?: string) =>
    tauriInvoke<string>('ai_request_with_model', { action, content, modelId, language }),
  cancelRequest: (requestId: string) => tauriInvoke<void>('ai_cancel_request', { requestId }),

  getChatHistory: () => tauriInvoke<AIChatMessage[]>('ai_get_chat_history'),
  saveChatHistory: (messages: AIChatMessage[]) =>
    tauriInvoke<void>('ai_save_chat_history', { messages }),
  clearChatHistory: () => tauriInvoke<void>('ai_clear_chat_history'),

  getTerminalChatHistory: (connectionId: string) =>
    tauriInvoke<AIChatMessage[]>('ai_get_terminal_chat_history', { connectionId }),
  saveTerminalChatHistory: (connectionId: string, messages: AIChatMessage[]) =>
    tauriInvoke<void>('ai_save_terminal_chat_history', { connectionId, messages }),
  clearTerminalChatHistory: (connectionId: string) =>
    tauriInvoke<void>('ai_clear_terminal_chat_history', { connectionId }),
};

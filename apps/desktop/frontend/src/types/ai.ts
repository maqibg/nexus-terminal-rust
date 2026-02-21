export type AIProviderType = 'openai' | 'anthropic' | 'gemini' | 'openai-compatible';

export type AIModelSourceType = 'auto' | 'manual';

export interface AIChannel {
  id: string;
  name: string;
  type: AIProviderType;
  apiKey: string;
  apiEndpoint?: string | null;
  enabled: boolean;
  createdAt: number;
  updatedAt: number;
}

export interface AIModel {
  id: string;
  modelId: string;
  displayName: string;
  channelId: string;
  contextWindow: number;
  type: AIModelSourceType;
  createdAt: number;
}

export interface AIConfigPrompts {
  explain: string;
  optimize: string;
  write: string;
}

export interface AIConfig {
  defaultModelId?: string;
  temperature: number;
  maxTokens: number;
  timeout: number;
  prompts: AIConfigPrompts;
}

export type AIAction = 'write' | 'explain' | 'optimize' | 'chat';

export interface AIChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  modelId?: string;
  status?: 'sending' | 'success' | 'error';
  error?: string;
}

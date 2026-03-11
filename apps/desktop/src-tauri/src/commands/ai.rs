//! AI assistant Tauri commands.

use std::{
    collections::HashMap,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use api_contract::error::{AppError, CmdResult};
use futures_util::StreamExt;
use reqwest::{header::CONTENT_TYPE, Client, StatusCode};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{json, Value};
use settings_core::repository::SettingsRepository;
use tauri::{Emitter, State};
use tokio::time::sleep;
use uuid::Uuid;

use crate::state::AppState;

const AI_CHANNELS_KEY: &str = "ai.channels";
const AI_MODELS_KEY: &str = "ai.models";
const AI_CONFIG_KEY: &str = "ai.config";
const AI_CHAT_HISTORY_KEY: &str = "ai.chat.history";
const AI_TERMINAL_CHAT_HISTORY_KEY: &str = "ai.chat.terminal.history";
const DEFAULT_PROMPT_WRITE: &str = "Write code based on this description: {content}\n\nLanguage: {language}\n\nReturn only the code without explanations or markdown code blocks.";
const DEFAULT_PROMPT_EXPLAIN: &str =
    "请作为一名资深开发人员，详细分析并解释以下代码片段的主要功能和目的。\n\n```{language}\n{content}\n```";
const DEFAULT_PROMPT_OPTIMIZE: &str = "Optimize this code:\n\n```{language}\n{content}\n```\n\nReturn only the optimized code without explanations or markdown code blocks.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AiProviderType {
    Openai,
    Anthropic,
    Gemini,
    OpenaiCompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiModelSourceType {
    Auto,
    Manual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AiAction {
    Write,
    Explain,
    Optimize,
    Chat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChannel {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: AiProviderType,
    pub api_key: String,
    pub api_endpoint: Option<String>,
    pub enabled: bool,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiModel {
    pub id: String,
    pub model_id: String,
    pub display_name: String,
    pub channel_id: String,
    pub context_window: i32,
    #[serde(rename = "type")]
    pub source_type: AiModelSourceType,
    pub created_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AiPrompts {
    pub explain: String,
    pub optimize: String,
    pub write: String,
}

impl Default for AiPrompts {
    fn default() -> Self {
        Self {
            explain: DEFAULT_PROMPT_EXPLAIN.to_string(),
            optimize: DEFAULT_PROMPT_OPTIMIZE.to_string(),
            write: DEFAULT_PROMPT_WRITE.to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AiConfig {
    pub default_model_id: Option<String>,
    pub temperature: f64,
    pub max_tokens: i32,
    pub timeout: i32,
    pub prompts: AiPrompts,
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            default_model_id: None,
            temperature: 0.7,
            max_tokens: 4000,
            timeout: 60000,
            prompts: AiPrompts::default(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddChannelRequest {
    pub name: String,
    #[serde(rename = "type")]
    pub provider_type: AiProviderType,
    pub api_key: String,
    pub api_endpoint: Option<String>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateChannelRequest {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub provider_type: Option<AiProviderType>,
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    pub enabled: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddModelRequest {
    pub model_id: String,
    pub display_name: String,
    pub channel_id: String,
    pub context_window: i32,
    #[serde(rename = "type")]
    pub source_type: AiModelSourceType,
}

#[derive(Debug, Clone, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AiConfigUpdate {
    pub default_model_id: Option<String>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i32>,
    pub timeout: Option<i32>,
    pub prompts: Option<AiPrompts>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AiChatMessage {
    pub id: String,
    pub role: String,
    pub content: String,
    pub reasoning_content: Option<String>,
    pub thinking_seconds: Option<f64>,
    pub timestamp: i64,
    pub model_id: Option<String>,
    pub status: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "lowercase")]
enum AiStreamChunkKind {
    Content,
    Reasoning,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiStreamChunkPayload {
    request_id: String,
    chunk: String,
    kind: AiStreamChunkKind,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiStreamCompletePayload {
    request_id: String,
    response: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiStreamErrorPayload {
    request_id: String,
    error: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct AiStreamCancelledPayload {
    request_id: String,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as i64
}

fn normalize_endpoint(endpoint: Option<&str>, provider: AiProviderType) -> String {
    let default = match provider {
        AiProviderType::Openai => "https://api.openai.com/v1",
        AiProviderType::Anthropic => "https://api.anthropic.com/v1",
        AiProviderType::Gemini => "https://generativelanguage.googleapis.com/v1beta",
        AiProviderType::OpenaiCompatible => "https://api.openai.com/v1",
    };

    endpoint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(default)
        .trim_end_matches('/')
        .to_string()
}

fn build_openai_models_url(endpoint: &str) -> String {
    if endpoint.ends_with("/v1") {
        format!("{endpoint}/models")
    } else {
        format!("{endpoint}/v1/models")
    }
}

fn build_openai_chat_url(endpoint: &str) -> String {
    if endpoint.ends_with("/v1") {
        format!("{endpoint}/chat/completions")
    } else {
        format!("{endpoint}/v1/chat/completions")
    }
}

fn infer_openai_context_window(model_id: &str) -> i32 {
    let lower = model_id.to_ascii_lowercase();

    if lower.contains("gpt-4.1") {
        return 128_000;
    }
    if lower.contains("gpt-4o") {
        return 128_000;
    }
    if lower.contains("gpt-4-turbo") {
        return 128_000;
    }
    if lower.contains("gpt-4-32k") {
        return 32_768;
    }
    if lower.contains("gpt-4") {
        return 8_192;
    }
    if lower.contains("gpt-3.5-turbo-16k") {
        return 16_384;
    }
    if lower.contains("gpt-3.5") {
        return 4_096;
    }
    if lower.contains("o1") || lower.contains("o3") {
        return 200_000;
    }

    4_096
}

fn map_openai_display_name(model_id: &str) -> String {
    match model_id {
        "gpt-4" => "GPT-4".into(),
        "gpt-4-turbo" => "GPT-4 Turbo".into(),
        "gpt-4o" => "GPT-4o".into(),
        "gpt-4o-mini" => "GPT-4o Mini".into(),
        "gpt-3.5-turbo" => "GPT-3.5 Turbo".into(),
        _ => model_id.to_string(),
    }
}

fn parse_int(value: Option<&Value>, fallback: i32) -> i32 {
    value
        .and_then(Value::as_i64)
        .and_then(|number| i32::try_from(number).ok())
        .unwrap_or(fallback)
}

fn extract_error_message(value: &Value, fallback: impl AsRef<str>) -> String {
    if let Some(message) = value
        .pointer("/error/message")
        .and_then(Value::as_str)
        .filter(|message| !message.trim().is_empty())
    {
        return message.to_string();
    }

    if let Some(message) = value
        .get("message")
        .and_then(Value::as_str)
        .filter(|message| !message.trim().is_empty())
    {
        return message.to_string();
    }

    if let Some(text) = value.as_str().filter(|text| !text.trim().is_empty()) {
        return text.to_string();
    }

    fallback.as_ref().to_string()
}

fn extract_text_from_response(value: &Value) -> Option<String> {
    if let Some(text) = value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_string());
    }

    if let Some(parts) = value
        .pointer("/choices/0/message/content")
        .and_then(Value::as_array)
    {
        let mut output = String::new();
        for item in parts {
            if let Some(text) = item
                .get("text")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
            {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(text);
            }
        }
        if !output.trim().is_empty() {
            return Some(output);
        }
    }

    if let Some(text) = value
        .pointer("/content/0/text")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
    {
        return Some(text.to_string());
    }

    if let Some(parts) = value
        .pointer("/candidates/0/content/parts")
        .and_then(Value::as_array)
    {
        let mut output = String::new();
        for item in parts {
            if let Some(text) = item
                .get("text")
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|text| !text.is_empty())
            {
                if !output.is_empty() {
                    output.push('\n');
                }
                output.push_str(text);
            }
        }
        if !output.trim().is_empty() {
            return Some(output);
        }
    }

    None
}

fn strip_markdown_code_fences(content: &str) -> String {
    let mut trimmed = content.trim().to_string();
    if !trimmed.starts_with("```") {
        return trimmed;
    }

    let lines: Vec<&str> = trimmed.lines().collect();
    if lines.is_empty() {
        return trimmed;
    }

    let mut start_index = 0usize;
    let mut end_index = lines.len();

    if lines
        .first()
        .is_some_and(|line| line.trim_start().starts_with("```"))
    {
        start_index = 1;
    }

    if lines
        .last()
        .is_some_and(|line| line.trim_end().starts_with("```"))
    {
        end_index = end_index.saturating_sub(1);
    }

    if start_index >= end_index {
        return trimmed;
    }

    trimmed = lines[start_index..end_index].join("\n");
    trimmed.trim().to_string()
}

fn split_text_chunks(text: &str, max_chars: usize) -> Vec<String> {
    if text.is_empty() {
        return Vec::new();
    }

    let chunk_size = max_chars.max(1);
    let mut chunks = Vec::new();
    let mut current = String::new();
    let mut current_len = 0usize;

    for ch in text.chars() {
        current.push(ch);
        current_len += 1;
        if current_len >= chunk_size {
            chunks.push(std::mem::take(&mut current));
            current_len = 0;
        }
    }

    if !current.is_empty() {
        chunks.push(current);
    }

    chunks
}

async fn register_cancel_flag(state: &AppState, request_id: &str) -> Arc<AtomicBool> {
    let cancel_flag = Arc::new(AtomicBool::new(false));
    let mut flags = state.ai_cancel_flags.lock().await;
    flags.insert(request_id.to_string(), cancel_flag.clone());
    cancel_flag
}

async fn remove_cancel_flag(state: &AppState, request_id: &str) {
    let mut flags = state.ai_cancel_flags.lock().await;
    flags.remove(request_id);
}

fn emit_ai_error(app_handle: &tauri::AppHandle, request_id: &str, message: &str) {
    let _ = app_handle.emit(
        "ai:error",
        AiStreamErrorPayload {
            request_id: request_id.to_string(),
            error: message.to_string(),
        },
    );
}

fn emit_ai_cancelled(app_handle: &tauri::AppHandle, request_id: &str) {
    let _ = app_handle.emit(
        "ai:cancelled",
        AiStreamCancelledPayload {
            request_id: request_id.to_string(),
        },
    );
}

fn emit_ai_chunk(
    app_handle: &tauri::AppHandle,
    request_id: &str,
    chunk: String,
    kind: AiStreamChunkKind,
) {
    let _ = app_handle.emit(
        "ai:stream-chunk",
        AiStreamChunkPayload {
            request_id: request_id.to_string(),
            chunk,
            kind,
        },
    );
}

async fn stream_response_chunks(
    app_handle: &tauri::AppHandle,
    request_id: &str,
    response: &str,
    cancel_flag: &Arc<AtomicBool>,
) -> CmdResult<()> {
    for chunk in split_text_chunks(response, 160) {
        if cancel_flag.load(Ordering::Relaxed) {
            emit_ai_cancelled(app_handle, request_id);
            return Err(AppError::Validation("请求已取消".into()));
        }

        let _ = app_handle.emit(
            "ai:stream-chunk",
            AiStreamChunkPayload {
                request_id: request_id.to_string(),
                chunk,
                kind: AiStreamChunkKind::Content,
            },
        );
        sleep(Duration::from_millis(10)).await;
    }

    let _ = app_handle.emit(
        "ai:complete",
        AiStreamCompletePayload {
            request_id: request_id.to_string(),
            response: response.to_string(),
        },
    );
    Ok(())
}

fn build_prompt(
    action: AiAction,
    content: &str,
    language: Option<&str>,
    config: &AiConfig,
) -> String {
    let language = language.unwrap_or("unknown");

    let write_template = if config.prompts.write.trim().is_empty() {
        DEFAULT_PROMPT_WRITE
    } else {
        config.prompts.write.as_str()
    };

    let explain_template = if config.prompts.explain.trim().is_empty() {
        DEFAULT_PROMPT_EXPLAIN
    } else {
        config.prompts.explain.as_str()
    };

    let optimize_template = if config.prompts.optimize.trim().is_empty() {
        DEFAULT_PROMPT_OPTIMIZE
    } else {
        config.prompts.optimize.as_str()
    };

    let template = match action {
        AiAction::Write => write_template,
        AiAction::Explain => explain_template,
        AiAction::Optimize => optimize_template,
        AiAction::Chat => "{content}",
    };

    template
        .replace("{content}", content)
        .replace("{language}", language)
}

fn build_http_client(timeout_ms: i32) -> CmdResult<Client> {
    let clamped = timeout_ms.clamp(5_000, 120_000);
    Client::builder()
        .timeout(Duration::from_millis(clamped as u64))
        .build()
        .map_err(|error| AppError::Internal(format!("创建 HTTP 客户端失败: {error}")))
}

async fn parse_json_response(response: reqwest::Response) -> CmdResult<(StatusCode, Value)> {
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|error| AppError::Internal(format!("读取 AI 响应失败: {error}")))?;

    if body.trim().is_empty() {
        return Ok((status, Value::Null));
    }

    match serde_json::from_str::<Value>(&body) {
        Ok(value) => Ok((status, value)),
        Err(_) => Ok((status, Value::String(body))),
    }
}

async fn load_json_setting<T>(state: &AppState, key: &str) -> CmdResult<Option<T>>
where
    T: DeserializeOwned,
{
    let raw = state
        .settings_repo
        .get_setting(key)
        .await
        .map_err(AppError::from)?;

    if let Some(raw) = raw {
        if raw.trim().is_empty() {
            return Ok(None);
        }

        let parsed = serde_json::from_str::<T>(&raw)
            .map_err(|error| AppError::Internal(format!("解析配置 {key} 失败: {error}")))?;
        return Ok(Some(parsed));
    }

    Ok(None)
}

async fn save_json_setting<T>(state: &AppState, key: &str, value: &T) -> CmdResult<()>
where
    T: Serialize + ?Sized,
{
    let serialized = serde_json::to_string(value)
        .map_err(|error| AppError::Internal(format!("序列化配置 {key} 失败: {error}")))?;

    state
        .settings_repo
        .set_setting(key, &serialized)
        .await
        .map_err(AppError::from)
}

async fn load_channels(state: &AppState) -> CmdResult<Vec<AiChannel>> {
    let mut channels = load_json_setting::<Vec<AiChannel>>(state, AI_CHANNELS_KEY)
        .await?
        .unwrap_or_default();

    for channel in &mut channels {
        if channel.api_key.trim().is_empty() {
            continue;
        }
        if let Ok(decrypted) = state.crypto.decrypt(&channel.api_key) {
            channel.api_key = decrypted;
        }
    }

    Ok(channels)
}

async fn save_channels(state: &AppState, channels: &[AiChannel]) -> CmdResult<()> {
    let mut encrypted_channels = channels.to_vec();
    for channel in &mut encrypted_channels {
        if channel.api_key.trim().is_empty() {
            continue;
        }
        channel.api_key = state
            .crypto
            .encrypt(&channel.api_key)
            .map_err(|error| AppError::Crypto(error.to_string()))?;
    }

    save_json_setting(state, AI_CHANNELS_KEY, &encrypted_channels).await
}

async fn load_models(state: &AppState) -> CmdResult<Vec<AiModel>> {
    Ok(load_json_setting::<Vec<AiModel>>(state, AI_MODELS_KEY)
        .await?
        .unwrap_or_default())
}

async fn save_models(state: &AppState, models: &[AiModel]) -> CmdResult<()> {
    save_json_setting(state, AI_MODELS_KEY, models).await
}

async fn load_config(state: &AppState) -> CmdResult<AiConfig> {
    let mut config = load_json_setting::<AiConfig>(state, AI_CONFIG_KEY)
        .await?
        .unwrap_or_default();

    if config.prompts.explain.trim().is_empty() {
        config.prompts.explain = DEFAULT_PROMPT_EXPLAIN.to_string();
    }
    if config.prompts.optimize.trim().is_empty() {
        config.prompts.optimize = DEFAULT_PROMPT_OPTIMIZE.to_string();
    }
    if config.prompts.write.trim().is_empty() {
        config.prompts.write = DEFAULT_PROMPT_WRITE.to_string();
    }

    Ok(config)
}

async fn save_config(state: &AppState, config: &AiConfig) -> CmdResult<()> {
    save_json_setting(state, AI_CONFIG_KEY, config).await
}

async fn load_chat_history(state: &AppState) -> CmdResult<Vec<AiChatMessage>> {
    Ok(
        load_json_setting::<Vec<AiChatMessage>>(state, AI_CHAT_HISTORY_KEY)
            .await?
            .unwrap_or_default(),
    )
}

async fn save_chat_history(state: &AppState, messages: &[AiChatMessage]) -> CmdResult<()> {
    save_json_setting(state, AI_CHAT_HISTORY_KEY, messages).await
}

async fn load_terminal_chat_history(
    state: &AppState,
) -> CmdResult<HashMap<String, Vec<AiChatMessage>>> {
    Ok(load_json_setting::<HashMap<String, Vec<AiChatMessage>>>(
        state,
        AI_TERMINAL_CHAT_HISTORY_KEY,
    )
    .await?
    .unwrap_or_default())
}

async fn save_terminal_chat_history(
    state: &AppState,
    history: &HashMap<String, Vec<AiChatMessage>>,
) -> CmdResult<()> {
    save_json_setting(state, AI_TERMINAL_CHAT_HISTORY_KEY, history).await
}

fn normalize_model_identifier(channel_id: &str, model_id: &str) -> String {
    format!("{channel_id}::{model_id}")
}

async fn fetch_models_from_openai_family(
    channel: &AiChannel,
    timeout: i32,
) -> CmdResult<Vec<AiModel>> {
    let endpoint = normalize_endpoint(channel.api_endpoint.as_deref(), channel.provider_type);
    let models_url = build_openai_models_url(&endpoint);
    let client = build_http_client(timeout)?;

    let response = client
        .get(models_url)
        .bearer_auth(&channel.api_key)
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("获取模型失败: {error}")))?;

    let (status, value) = parse_json_response(response).await?;
    if !status.is_success() {
        return Err(AppError::Internal(format!(
            "获取模型失败({status}): {}",
            extract_error_message(&value, "请求失败")
        )));
    }

    let source_list = if let Some(array) = value.get("data").and_then(Value::as_array) {
        array.clone()
    } else if let Some(array) = value.as_array() {
        array.clone()
    } else if let Some(array) = value.get("models").and_then(Value::as_array) {
        array.clone()
    } else {
        Vec::new()
    };

    let mut models = Vec::new();
    for item in source_list {
        let model_id = item
            .get("id")
            .and_then(Value::as_str)
            .map(str::trim)
            .unwrap_or_default();
        if model_id.is_empty() {
            continue;
        }

        let display_name = item
            .get("display_name")
            .or_else(|| item.get("displayName"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| map_openai_display_name(model_id));

        let context_window = parse_int(
            item.get("context_length")
                .or_else(|| item.get("context_window"))
                .or_else(|| item.get("contextWindow"))
                .or_else(|| item.get("input_token_limit")),
            infer_openai_context_window(model_id),
        );

        models.push(AiModel {
            id: normalize_model_identifier(&channel.id, model_id),
            model_id: model_id.to_string(),
            display_name,
            channel_id: channel.id.clone(),
            context_window,
            source_type: AiModelSourceType::Auto,
            created_at: now_ms(),
        });
    }

    Ok(models)
}

async fn fetch_models_from_anthropic(channel: &AiChannel, timeout: i32) -> CmdResult<Vec<AiModel>> {
    let endpoint = normalize_endpoint(channel.api_endpoint.as_deref(), channel.provider_type);
    let url = if endpoint.ends_with("/v1") {
        format!("{endpoint}/models")
    } else {
        format!("{endpoint}/v1/models")
    };

    let client = build_http_client(timeout)?;
    let response = client
        .get(url)
        .header("x-api-key", &channel.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("获取模型失败: {error}")))?;

    let (status, value) = parse_json_response(response).await?;
    if !status.is_success() {
        return Err(AppError::Internal(format!(
            "获取模型失败({status}): {}",
            extract_error_message(&value, "请求失败")
        )));
    }

    let mut models = Vec::new();
    if let Some(items) = value.get("data").and_then(Value::as_array) {
        for item in items {
            let model_id = item
                .get("id")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default();
            if model_id.is_empty() {
                continue;
            }

            let display_name = item
                .get("display_name")
                .or_else(|| item.get("displayName"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|name| !name.is_empty())
                .unwrap_or(model_id)
                .to_string();

            let context_window = parse_int(
                item.get("context_window")
                    .or_else(|| item.get("contextWindow"))
                    .or_else(|| item.get("input_token_limit")),
                200_000,
            );

            models.push(AiModel {
                id: normalize_model_identifier(&channel.id, model_id),
                model_id: model_id.to_string(),
                display_name,
                channel_id: channel.id.clone(),
                context_window,
                source_type: AiModelSourceType::Auto,
                created_at: now_ms(),
            });
        }
    }

    Ok(models)
}

async fn fetch_models_from_gemini(channel: &AiChannel, timeout: i32) -> CmdResult<Vec<AiModel>> {
    let endpoint = normalize_endpoint(channel.api_endpoint.as_deref(), channel.provider_type);
    let url = format!("{endpoint}/models?key={}", channel.api_key);

    let client = build_http_client(timeout)?;
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("获取模型失败: {error}")))?;

    let (status, value) = parse_json_response(response).await?;
    if !status.is_success() {
        return Err(AppError::Internal(format!(
            "获取模型失败({status}): {}",
            extract_error_message(&value, "请求失败")
        )));
    }

    let mut models = Vec::new();
    if let Some(items) = value.get("models").and_then(Value::as_array) {
        for item in items {
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .map(str::trim)
                .unwrap_or_default();

            if name.is_empty() {
                continue;
            }

            let model_id = name.strip_prefix("models/").unwrap_or(name);
            if !model_id.contains("gemini") {
                continue;
            }

            let display_name = item
                .get("displayName")
                .or_else(|| item.get("display_name"))
                .and_then(Value::as_str)
                .map(str::trim)
                .filter(|label| !label.is_empty())
                .unwrap_or(model_id)
                .to_string();

            let context_window = parse_int(
                item.get("inputTokenLimit")
                    .or_else(|| item.get("input_token_limit"))
                    .or_else(|| item.get("context_window")),
                32_768,
            );

            models.push(AiModel {
                id: normalize_model_identifier(&channel.id, model_id),
                model_id: model_id.to_string(),
                display_name,
                channel_id: channel.id.clone(),
                context_window,
                source_type: AiModelSourceType::Auto,
                created_at: now_ms(),
            });
        }
    }

    Ok(models)
}

async fn fetch_models_by_channel(channel: &AiChannel, timeout: i32) -> CmdResult<Vec<AiModel>> {
    match channel.provider_type {
        AiProviderType::Openai | AiProviderType::OpenaiCompatible => {
            fetch_models_from_openai_family(channel, timeout).await
        }
        AiProviderType::Anthropic => fetch_models_from_anthropic(channel, timeout).await,
        AiProviderType::Gemini => fetch_models_from_gemini(channel, timeout).await,
    }
}

async fn verify_channel_connectivity(channel: &AiChannel, timeout: i32) -> bool {
    fetch_models_by_channel(channel, timeout).await.is_ok()
}

async fn request_openai_family(
    channel: &AiChannel,
    model: &AiModel,
    config: &AiConfig,
    prompt: &str,
) -> CmdResult<String> {
    let endpoint = normalize_endpoint(channel.api_endpoint.as_deref(), channel.provider_type);
    let url = build_openai_chat_url(&endpoint);
    let client = build_http_client(config.timeout)?;

    let mut body = json!({
        "model": model.model_id,
        "messages": [
            {
                "role": "user",
                "content": prompt,
            }
        ],
    });

    if model.model_id.starts_with("o1") || model.model_id.starts_with("o3") {
        body["max_completion_tokens"] = json!(config.max_tokens);
        body["temperature"] = json!(1.0);
    } else {
        body["max_tokens"] = json!(config.max_tokens);
        body["temperature"] = json!(config.temperature);
    }

    let response = client
        .post(url)
        .bearer_auth(&channel.api_key)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("AI 请求失败: {error}")))?;

    let (status, value) = parse_json_response(response).await?;
    if !status.is_success() {
        return Err(AppError::Internal(format!(
            "AI 请求失败({status}): {}",
            extract_error_message(&value, "请求失败")
        )));
    }

    extract_text_from_response(&value)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| AppError::Internal("AI 返回了空响应".into()))
}

fn is_event_stream_response(response: &reqwest::Response) -> bool {
    response
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value.to_ascii_lowercase().contains("text/event-stream"))
}

fn looks_like_json_line(source: &str) -> bool {
    let trimmed = source.trim();
    (trimmed.starts_with('{') && trimmed.ends_with('}'))
        || (trimmed.starts_with('[') && trimmed.ends_with(']'))
}

fn extract_text_from_openai_content_array(parts: &[Value]) -> Option<String> {
    let mut output = String::new();
    for item in parts {
        let text = item
            .get("text")
            .or_else(|| item.get("content"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .or_else(|| item.as_str().map(str::trim).filter(|text| !text.is_empty()));

        if let Some(text) = text {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(text);
        }
    }

    if output.trim().is_empty() {
        None
    } else {
        Some(output)
    }
}

fn extract_openai_stream_reasoning(value: &Value) -> Option<String> {
    value
        .pointer("/choices/0/delta/reasoning_content")
        .or_else(|| value.pointer("/choices/0/delta/reasoning"))
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn extract_openai_stream_content(value: &Value) -> Option<String> {
    if let Some(text) = value
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_str)
    {
        return Some(text.to_string());
    }

    if let Some(parts) = value
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_array)
    {
        if let Some(text) = extract_text_from_openai_content_array(parts) {
            return Some(text);
        }
    }

    if let Some(text) = value
        .pointer("/choices/0/delta/text")
        .and_then(Value::as_str)
    {
        return Some(text.to_string());
    }

    if let Some(text) = value.pointer("/choices/0/text").and_then(Value::as_str) {
        return Some(text.to_string());
    }

    extract_text_from_response(value)
}

fn openai_stream_has_finish_reason(value: &Value) -> bool {
    let finish = value
        .pointer("/choices/0/finish_reason")
        .or_else(|| value.pointer("/choices/0/finishReason"))
        .or_else(|| value.pointer("/choices/0/delta/finish_reason"))
        .or_else(|| value.pointer("/choices/0/delta/finishReason"));

    match finish {
        Some(Value::String(reason)) => {
            let trimmed = reason.trim();
            !trimmed.is_empty() && trimmed != "null"
        }
        _ => false,
    }
}

const HIDDEN_THOUGHT_START_TAGS: [&str; 2] = ["<think>", "<analysis>"];
const HIDDEN_THOUGHT_END_TAGS: [&str; 2] = ["</think>", "</analysis>"];
const HIDDEN_THOUGHT_MAX_TAG_BYTES: usize = 10;

fn find_first_tag(source: &str, tags: &[&'static str]) -> Option<(usize, &'static str)> {
    tags.iter()
        .filter_map(|tag| source.find(*tag).map(|pos| (pos, *tag)))
        .min_by_key(|(pos, _)| *pos)
}

fn find_next_hidden_thought_tag(source: &str) -> Option<(usize, &'static str, bool)> {
    let start = find_first_tag(source, &HIDDEN_THOUGHT_START_TAGS).map(|(pos, tag)| (pos, tag, true));
    let end = find_first_tag(source, &HIDDEN_THOUGHT_END_TAGS).map(|(pos, tag)| (pos, tag, false));

    match (start, end) {
        (Some(start), Some(end)) => {
            if start.0 <= end.0 {
                Some(start)
            } else {
                Some(end)
            }
        }
        (Some(start), None) => Some(start),
        (None, Some(end)) => Some(end),
        (None, None) => None,
    }
}

fn split_hidden_thought_blocks_with_state(source: &str, in_block: &mut bool) -> (String, String) {
    let mut remaining = source;
    let mut visible = String::new();
    let mut hidden = String::new();

    loop {
        let Some((pos, tag, is_start)) = find_next_hidden_thought_tag(remaining) else {
            if *in_block {
                hidden.push_str(remaining);
            } else {
                visible.push_str(remaining);
            }
            break;
        };

        if *in_block {
            hidden.push_str(&remaining[..pos]);
            remaining = &remaining[pos + tag.len()..];
            if !is_start {
                *in_block = false;
            }
            continue;
        }

        if is_start {
            visible.push_str(&remaining[..pos]);
            remaining = &remaining[pos + tag.len()..];
            *in_block = true;
            continue;
        }

        hidden.push_str(&remaining[..pos]);
        remaining = &remaining[pos + tag.len()..];
    }

    (visible, hidden)
}

fn strip_hidden_thought_blocks_with_state(source: &str, in_block: &mut bool) -> String {
    split_hidden_thought_blocks_with_state(source, in_block).0
}

fn strip_hidden_thought_blocks(source: &str) -> String {
    let mut in_block = false;
    strip_hidden_thought_blocks_with_state(source, &mut in_block)
}

#[derive(Debug, Default)]
struct HiddenThoughtStreamFilter {
    in_block: bool,
    carry: String,
}

impl HiddenThoughtStreamFilter {
    fn split_index_with_guard(&self, source: &str, min_tail: usize) -> usize {
        let Some(mut split_index) = source.len().checked_sub(min_tail) else {
            return 0;
        };

        while split_index > 0 && !source.is_char_boundary(split_index) {
            split_index -= 1;
        }

        if split_index == 0 {
            return 0;
        }

        let bytes = source.as_bytes();
        let window_start = split_index.saturating_sub(HIDDEN_THOUGHT_MAX_TAG_BYTES);
        let mut adjust_to: Option<usize> = None;

        for pos in (window_start..split_index).rev() {
            if bytes[pos] != b'<' {
                continue;
            }

            let prefix = &bytes[pos..split_index];
            let matches = HIDDEN_THOUGHT_START_TAGS
                .iter()
                .any(|tag| tag.as_bytes().starts_with(prefix))
                || HIDDEN_THOUGHT_END_TAGS
                    .iter()
                    .any(|tag| tag.as_bytes().starts_with(prefix));

            if matches {
                adjust_to = Some(pos);
                break;
            }
        }

        if let Some(pos) = adjust_to {
            split_index = pos;
        }

        split_index
    }

    fn push(&mut self, input: &str) -> (String, String) {
        let combined = if self.carry.is_empty() {
            input.to_string()
        } else {
            let mut merged = std::mem::take(&mut self.carry);
            merged.push_str(input);
            merged
        };

        if combined.is_empty() {
            return (String::new(), String::new());
        }

        if combined.len() <= HIDDEN_THOUGHT_MAX_TAG_BYTES {
            self.carry = combined;
            return (String::new(), String::new());
        }

        let split_index = self.split_index_with_guard(&combined, HIDDEN_THOUGHT_MAX_TAG_BYTES);

        let (processable, tail) = combined.split_at(split_index);
        self.carry = tail.to_string();
        split_hidden_thought_blocks_with_state(processable, &mut self.in_block)
    }

    fn finish(&mut self) -> (String, String) {
        if self.carry.is_empty() {
            return (String::new(), String::new());
        }
        let remaining = std::mem::take(&mut self.carry);
        split_hidden_thought_blocks_with_state(&remaining, &mut self.in_block)
    }
}

fn handle_openai_stream_payload(
    app_handle: &tauri::AppHandle,
    request_id: &str,
    payload: &str,
    thought_filter: &mut HiddenThoughtStreamFilter,
    reasoning_filter: &mut HiddenThoughtStreamFilter,
    output: &mut String,
    reasoning_output: &mut String,
) -> CmdResult<bool> {
    let trimmed = payload.trim();
    if trimmed.is_empty() {
        return Ok(false);
    }

    if trimmed == "[DONE]" {
        return Ok(true);
    }

    let value: Value = serde_json::from_str(trimmed)
        .map_err(|error| AppError::Internal(format!("解析 AI 流失败: {error}")))?;

    if let Some(error) = value.get("error") {
        return Err(AppError::Internal(format!(
            "AI 请求失败: {}",
            extract_error_message(error, "请求失败")
        )));
    }

    if let Some(reasoning) = extract_openai_stream_reasoning(&value).filter(|text| !text.is_empty())
    {
        let (visible, hidden) = reasoning_filter.push(&reasoning);
        if !hidden.is_empty() {
            reasoning_output.push_str(&hidden);
            emit_ai_chunk(app_handle, request_id, hidden, AiStreamChunkKind::Reasoning);
        }
        if !visible.is_empty() {
            output.push_str(&visible);
            emit_ai_chunk(app_handle, request_id, visible, AiStreamChunkKind::Content);
        }
    }

    if let Some(content) = extract_openai_stream_content(&value).filter(|text| !text.is_empty()) {
        let (visible, hidden) = thought_filter.push(&content);
        if !hidden.is_empty() {
            reasoning_output.push_str(&hidden);
            emit_ai_chunk(app_handle, request_id, hidden, AiStreamChunkKind::Reasoning);
        }
        if !visible.is_empty() {
            output.push_str(&visible);
            emit_ai_chunk(app_handle, request_id, visible, AiStreamChunkKind::Content);
        }
    }

    if openai_stream_has_finish_reason(&value) {
        return Ok(true);
    }

    Ok(false)
}

async fn stream_openai_sse_response(
    app_handle: &tauri::AppHandle,
    request_id: &str,
    response: reqwest::Response,
    cancel_flag: &Arc<AtomicBool>,
) -> CmdResult<String> {
    let mut output = String::new();
    let mut reasoning_output = String::new();
    let mut buffer: Vec<u8> = Vec::new();
    let mut pending_data: Vec<String> = Vec::new();
    let mut done = false;
    let mut thought_filter = HiddenThoughtStreamFilter::default();
    let mut reasoning_filter = HiddenThoughtStreamFilter {
        in_block: true,
        carry: String::new(),
    };

    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        if cancel_flag.load(Ordering::Relaxed) {
            emit_ai_cancelled(app_handle, request_id);
            return Err(AppError::Validation("请求已取消".into()));
        }

        let bytes =
            chunk.map_err(|error| AppError::Internal(format!("读取 AI 流失败: {error}")))?;
        buffer.extend_from_slice(&bytes);

        while let Some(newline) = buffer.iter().position(|item| *item == b'\n') {
            let mut line_bytes: Vec<u8> = buffer.drain(..=newline).collect();
            if line_bytes.last() == Some(&b'\n') {
                line_bytes.pop();
            }
            if line_bytes.last() == Some(&b'\r') {
                line_bytes.pop();
            }

            let line = match String::from_utf8(line_bytes) {
                Ok(value) => value,
                Err(error) => String::from_utf8_lossy(&error.into_bytes()).into_owned(),
            };

            if line.is_empty() {
                if pending_data.is_empty() {
                    continue;
                }

                let payload = pending_data.join("\n");
                pending_data.clear();
                if handle_openai_stream_payload(
                    app_handle,
                    request_id,
                    &payload,
                    &mut thought_filter,
                    &mut reasoning_filter,
                    &mut output,
                    &mut reasoning_output,
                )? {
                    done = true;
                    break;
                }
                continue;
            }

            if line.starts_with(':') {
                continue;
            }

            if let Some(data) = line.strip_prefix("data:") {
                let trimmed = data.trim_start();
                pending_data.push(trimmed.to_string());
                if trimmed.trim() == "[DONE]"
                    || (pending_data.len() == 1 && looks_like_json_line(trimmed))
                {
                    let payload = pending_data.join("\n");
                    pending_data.clear();
                    if handle_openai_stream_payload(
                        app_handle,
                        request_id,
                        &payload,
                        &mut thought_filter,
                        &mut reasoning_filter,
                        &mut output,
                        &mut reasoning_output,
                    )? {
                        done = true;
                        break;
                    }
                }
                continue;
            }

            if pending_data.is_empty() && looks_like_json_line(&line) {
                if handle_openai_stream_payload(
                    app_handle,
                    request_id,
                    &line,
                    &mut thought_filter,
                    &mut reasoning_filter,
                    &mut output,
                    &mut reasoning_output,
                )? {
                    done = true;
                    break;
                }
            }
        }

        if done {
            break;
        }
    }

    if cancel_flag.load(Ordering::Relaxed) {
        emit_ai_cancelled(app_handle, request_id);
        return Err(AppError::Validation("请求已取消".into()));
    }

    if !pending_data.is_empty() && !done {
        let payload = pending_data.join("\n");
        pending_data.clear();
        if let Err(error) = handle_openai_stream_payload(
            app_handle,
            request_id,
            &payload,
            &mut thought_filter,
            &mut reasoning_filter,
            &mut output,
            &mut reasoning_output,
        ) {
            if output.trim().is_empty() && reasoning_output.trim().is_empty() {
                return Err(error);
            }
        }
    }

    let (reason_tail_visible, reason_tail_hidden) = reasoning_filter.finish();
    if !reason_tail_hidden.is_empty() {
        reasoning_output.push_str(&reason_tail_hidden);
        emit_ai_chunk(
            app_handle,
            request_id,
            reason_tail_hidden,
            AiStreamChunkKind::Reasoning,
        );
    }
    if !reason_tail_visible.is_empty() {
        output.push_str(&reason_tail_visible);
        emit_ai_chunk(
            app_handle,
            request_id,
            reason_tail_visible,
            AiStreamChunkKind::Content,
        );
    }

    let (tail_visible, tail_hidden) = thought_filter.finish();
    if !tail_hidden.is_empty() {
        reasoning_output.push_str(&tail_hidden);
        emit_ai_chunk(
            app_handle,
            request_id,
            tail_hidden,
            AiStreamChunkKind::Reasoning,
        );
    }
    if !tail_visible.is_empty() {
        output.push_str(&tail_visible);
        emit_ai_chunk(
            app_handle,
            request_id,
            tail_visible,
            AiStreamChunkKind::Content,
        );
    }

    if output.trim().is_empty() && reasoning_output.trim().is_empty() {
        return Err(AppError::Internal("AI 返回了空响应".into()));
    }

    let _ = app_handle.emit(
        "ai:complete",
        AiStreamCompletePayload {
            request_id: request_id.to_string(),
            response: output.clone(),
        },
    );

    Ok(output)
}

async fn request_openai_family_streaming(
    app_handle: &tauri::AppHandle,
    request_id: &str,
    cancel_flag: &Arc<AtomicBool>,
    channel: &AiChannel,
    model: &AiModel,
    config: &AiConfig,
    prompt: &str,
) -> CmdResult<String> {
    let endpoint = normalize_endpoint(channel.api_endpoint.as_deref(), channel.provider_type);
    let url = build_openai_chat_url(&endpoint);
    let client = build_http_client(config.timeout)?;

    let mut body = json!({
        "model": model.model_id,
        "stream": true,
        "messages": [
            {
                "role": "user",
                "content": prompt,
            }
        ],
    });

    if model.model_id.starts_with("o1") || model.model_id.starts_with("o3") {
        body["max_completion_tokens"] = json!(config.max_tokens);
        body["temperature"] = json!(1.0);
    } else {
        body["max_tokens"] = json!(config.max_tokens);
        body["temperature"] = json!(config.temperature);
    }

    let response = client
        .post(url)
        .bearer_auth(&channel.api_key)
        .header("content-type", "application/json")
        .header("accept", "text/event-stream")
        .json(&body)
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("AI 请求失败: {error}")))?;

    let status = response.status();
    if !status.is_success() {
        let (status, value) = parse_json_response(response).await?;
        return Err(AppError::Internal(format!(
            "AI 请求失败({status}): {}",
            extract_error_message(&value, "请求失败")
        )));
    }

    if !is_event_stream_response(&response) {
        let (_status, value) = parse_json_response(response).await?;
        let text = extract_text_from_response(&value)
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| AppError::Internal("AI 返回了空响应".into()))?;
        let cleaned = strip_hidden_thought_blocks(&text);
        stream_response_chunks(app_handle, request_id, &cleaned, cancel_flag).await?;
        return Ok(cleaned);
    }

    stream_openai_sse_response(app_handle, request_id, response, cancel_flag).await
}

async fn request_anthropic(
    channel: &AiChannel,
    model: &AiModel,
    config: &AiConfig,
    prompt: &str,
) -> CmdResult<String> {
    let endpoint = normalize_endpoint(channel.api_endpoint.as_deref(), channel.provider_type);
    let url = if endpoint.ends_with("/v1") {
        format!("{endpoint}/messages")
    } else {
        format!("{endpoint}/v1/messages")
    };

    let client = build_http_client(config.timeout)?;
    let body = json!({
        "model": model.model_id,
        "max_tokens": config.max_tokens,
        "temperature": config.temperature,
        "messages": [
            {
                "role": "user",
                "content": prompt,
            }
        ],
    });

    let response = client
        .post(url)
        .header("x-api-key", &channel.api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("AI 请求失败: {error}")))?;

    let (status, value) = parse_json_response(response).await?;
    if !status.is_success() {
        return Err(AppError::Internal(format!(
            "AI 请求失败({status}): {}",
            extract_error_message(&value, "请求失败")
        )));
    }

    extract_text_from_response(&value)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| AppError::Internal("AI 返回了空响应".into()))
}

async fn request_gemini(
    channel: &AiChannel,
    model: &AiModel,
    config: &AiConfig,
    prompt: &str,
) -> CmdResult<String> {
    let endpoint = normalize_endpoint(channel.api_endpoint.as_deref(), channel.provider_type);
    let model_path = if model.model_id.starts_with("models/") {
        model.model_id.clone()
    } else {
        format!("models/{}", model.model_id)
    };
    let url = format!(
        "{endpoint}/{model_path}:generateContent?key={}",
        channel.api_key
    );

    let client = build_http_client(config.timeout)?;
    let body = json!({
        "contents": [
            {
                "role": "user",
                "parts": [
                    { "text": prompt }
                ],
            }
        ],
        "generationConfig": {
            "temperature": config.temperature,
            "maxOutputTokens": config.max_tokens,
        },
    });

    let response = client
        .post(url)
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|error| AppError::Internal(format!("AI 请求失败: {error}")))?;

    let (status, value) = parse_json_response(response).await?;
    if !status.is_success() {
        return Err(AppError::Internal(format!(
            "AI 请求失败({status}): {}",
            extract_error_message(&value, "请求失败")
        )));
    }

    extract_text_from_response(&value)
        .filter(|text| !text.trim().is_empty())
        .ok_or_else(|| AppError::Internal("AI 返回了空响应".into()))
}

async fn request_model_response(
    channel: &AiChannel,
    model: &AiModel,
    config: &AiConfig,
    prompt: &str,
) -> CmdResult<String> {
    match channel.provider_type {
        AiProviderType::Openai | AiProviderType::OpenaiCompatible => {
            request_openai_family(channel, model, config, prompt).await
        }
        AiProviderType::Anthropic => request_anthropic(channel, model, config, prompt).await,
        AiProviderType::Gemini => request_gemini(channel, model, config, prompt).await,
    }
}

async fn perform_request_with_model(
    state: &AppState,
    app_handle: &tauri::AppHandle,
    action: AiAction,
    content: String,
    model_id: String,
    language: Option<String>,
    request_id: String,
) -> CmdResult<String> {
    let cancel_flag = register_cancel_flag(state, &request_id).await;

    let channels = load_channels(state).await?;
    let models = load_models(state).await?;
    let config = load_config(state).await?;

    let model = models
        .iter()
        .find(|item| item.id == model_id)
        .ok_or_else(|| AppError::NotFound("AI 模型不存在".into()))?;
    let channel = channels
        .iter()
        .find(|item| item.id == model.channel_id)
        .ok_or_else(|| AppError::NotFound("AI 渠道不存在".into()))?;

    if !channel.enabled {
        return Err(AppError::Validation("当前渠道已禁用，请先启用".into()));
    }

    let prompt = build_prompt(action, &content, language.as_deref(), &config);
    let mut response = if matches!(
        channel.provider_type,
        AiProviderType::Openai | AiProviderType::OpenaiCompatible
    ) {
        match request_openai_family_streaming(
            app_handle,
            &request_id,
            &cancel_flag,
            channel,
            model,
            &config,
            &prompt,
        )
        .await
        {
            Ok(value) => value,
            Err(error) => {
                if !matches!(&error, AppError::Validation(message) if message == "请求已取消")
                {
                    emit_ai_error(app_handle, &request_id, &error.to_string());
                }
                remove_cancel_flag(state, &request_id).await;
                return Err(error);
            }
        }
    } else {
        match request_model_response(channel, model, &config, &prompt).await {
            Ok(value) => value,
            Err(error) => {
                emit_ai_error(app_handle, &request_id, &error.to_string());
                remove_cancel_flag(state, &request_id).await;
                return Err(error);
            }
        }
    };

    if matches!(
        channel.provider_type,
        AiProviderType::Anthropic | AiProviderType::Gemini
    ) {
        if cancel_flag.load(Ordering::Relaxed) {
            emit_ai_cancelled(app_handle, &request_id);
            remove_cancel_flag(state, &request_id).await;
            return Err(AppError::Validation("请求已取消".into()));
        }

        if matches!(action, AiAction::Write | AiAction::Optimize) {
            response = strip_markdown_code_fences(&response);
        }

        if let Err(error) =
            stream_response_chunks(app_handle, &request_id, &response, &cancel_flag).await
        {
            if !matches!(&error, AppError::Validation(message) if message == "请求已取消") {
                emit_ai_error(app_handle, &request_id, &error.to_string());
            }
            remove_cancel_flag(state, &request_id).await;
            return Err(error);
        }
    }

    remove_cancel_flag(state, &request_id).await;
    Ok(response)
}

#[tauri::command]
pub async fn ai_get_all_channels(state: State<'_, AppState>) -> CmdResult<Vec<AiChannel>> {
    state.auth.require_auth().await?;
    load_channels(&state).await
}

#[tauri::command]
pub async fn ai_add_channel(
    state: State<'_, AppState>,
    req: AddChannelRequest,
) -> CmdResult<AiChannel> {
    state.auth.require_auth().await?;

    if req.name.trim().is_empty() {
        return Err(AppError::Validation("渠道名称不能为空".into()));
    }
    if req.api_key.trim().is_empty() {
        return Err(AppError::Validation("API Key 不能为空".into()));
    }

    let mut channels = load_channels(&state).await?;
    let now = now_ms();
    let channel = AiChannel {
        id: Uuid::new_v4().to_string(),
        name: req.name.trim().to_string(),
        provider_type: req.provider_type,
        api_key: req.api_key.trim().to_string(),
        api_endpoint: req
            .api_endpoint
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        enabled: req.enabled,
        created_at: now,
        updated_at: now,
    };

    channels.push(channel.clone());
    save_channels(&state, &channels).await?;
    Ok(channel)
}

#[tauri::command]
pub async fn ai_update_channel(
    state: State<'_, AppState>,
    id: String,
    updates: UpdateChannelRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;

    let mut channels = load_channels(&state).await?;
    let channel = channels
        .iter_mut()
        .find(|channel| channel.id == id)
        .ok_or_else(|| AppError::NotFound("AI 渠道不存在".into()))?;

    if let Some(name) = updates.name {
        let normalized = name.trim();
        if normalized.is_empty() {
            return Err(AppError::Validation("渠道名称不能为空".into()));
        }
        channel.name = normalized.to_string();
    }

    if let Some(provider_type) = updates.provider_type {
        channel.provider_type = provider_type;
    }

    if let Some(api_key) = updates.api_key {
        let normalized = api_key.trim();
        if !normalized.is_empty() {
            channel.api_key = normalized.to_string();
        }
    }

    if let Some(api_endpoint) = updates.api_endpoint {
        let normalized = api_endpoint.trim().to_string();
        channel.api_endpoint = if normalized.is_empty() {
            None
        } else {
            Some(normalized)
        };
    }

    if let Some(enabled) = updates.enabled {
        channel.enabled = enabled;
    }

    channel.updated_at = now_ms();
    save_channels(&state, &channels).await
}

#[tauri::command]
pub async fn ai_delete_channel(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.auth.require_auth().await?;

    let mut channels = load_channels(&state).await?;
    let mut models = load_models(&state).await?;
    let mut config = load_config(&state).await?;

    let original_count = channels.len();
    channels.retain(|channel| channel.id != id);
    if channels.len() == original_count {
        return Err(AppError::NotFound("AI 渠道不存在".into()));
    }

    models.retain(|model| model.channel_id != id);
    if let Some(default_model_id) = &config.default_model_id {
        if !models.iter().any(|model| &model.id == default_model_id) {
            config.default_model_id = None;
        }
    }

    save_channels(&state, &channels).await?;
    save_models(&state, &models).await?;
    save_config(&state, &config).await
}

#[tauri::command]
pub async fn ai_verify_channel(state: State<'_, AppState>, id: String) -> CmdResult<bool> {
    state.auth.require_auth().await?;

    let channels = load_channels(&state).await?;
    let channel = channels
        .iter()
        .find(|channel| channel.id == id)
        .ok_or_else(|| AppError::NotFound("AI 渠道不存在".into()))?;

    Ok(verify_channel_connectivity(channel, 20_000).await)
}

#[tauri::command]
pub async fn ai_fetch_models(
    state: State<'_, AppState>,
    channel_id: String,
) -> CmdResult<Vec<AiModel>> {
    state.auth.require_auth().await?;

    let channels = load_channels(&state).await?;
    let channel = channels
        .iter()
        .find(|channel| channel.id == channel_id)
        .ok_or_else(|| AppError::NotFound("AI 渠道不存在".into()))?;

    let mut models = load_models(&state).await?;
    let fetched_models = fetch_models_by_channel(channel, 30_000).await?;

    models.retain(|model| {
        !(model.channel_id == channel.id && model.source_type == AiModelSourceType::Auto)
    });
    models.extend(fetched_models.clone());
    save_models(&state, &models).await?;

    Ok(fetched_models)
}

#[tauri::command]
pub async fn ai_add_model(state: State<'_, AppState>, req: AddModelRequest) -> CmdResult<AiModel> {
    state.auth.require_auth().await?;

    if req.model_id.trim().is_empty() {
        return Err(AppError::Validation("模型 ID 不能为空".into()));
    }
    if req.display_name.trim().is_empty() {
        return Err(AppError::Validation("模型名称不能为空".into()));
    }

    let channels = load_channels(&state).await?;
    if !channels.iter().any(|channel| channel.id == req.channel_id) {
        return Err(AppError::NotFound("所属渠道不存在".into()));
    }

    let mut models = load_models(&state).await?;
    let model = AiModel {
        id: Uuid::new_v4().to_string(),
        model_id: req.model_id.trim().to_string(),
        display_name: req.display_name.trim().to_string(),
        channel_id: req.channel_id,
        context_window: req.context_window.clamp(1000, 1_000_000),
        source_type: req.source_type,
        created_at: now_ms(),
    };

    models.push(model.clone());
    save_models(&state, &models).await?;
    Ok(model)
}

#[tauri::command]
pub async fn ai_delete_model(state: State<'_, AppState>, id: String) -> CmdResult<()> {
    state.auth.require_auth().await?;

    let mut models = load_models(&state).await?;
    let mut config = load_config(&state).await?;

    let original_count = models.len();
    models.retain(|model| model.id != id);
    if models.len() == original_count {
        return Err(AppError::NotFound("AI 模型不存在".into()));
    }

    if config
        .default_model_id
        .as_ref()
        .is_some_and(|default_model_id| default_model_id == &id)
    {
        config.default_model_id = None;
    }

    save_models(&state, &models).await?;
    save_config(&state, &config).await
}

#[tauri::command]
pub async fn ai_get_all_models(state: State<'_, AppState>) -> CmdResult<Vec<AiModel>> {
    state.auth.require_auth().await?;
    load_models(&state).await
}

#[tauri::command]
pub async fn ai_set_default_model(state: State<'_, AppState>, model_id: String) -> CmdResult<()> {
    state.auth.require_auth().await?;

    let models = load_models(&state).await?;
    if !models.iter().any(|model| model.id == model_id) {
        return Err(AppError::NotFound("AI 模型不存在".into()));
    }

    let mut config = load_config(&state).await?;
    config.default_model_id = Some(model_id);
    save_config(&state, &config).await
}

#[tauri::command]
pub async fn ai_get_config(state: State<'_, AppState>) -> CmdResult<AiConfig> {
    state.auth.require_auth().await?;
    load_config(&state).await
}

#[tauri::command]
pub async fn ai_update_config(
    state: State<'_, AppState>,
    updates: AiConfigUpdate,
) -> CmdResult<()> {
    state.auth.require_auth().await?;

    let mut config = load_config(&state).await?;

    if let Some(default_model_id) = updates.default_model_id {
        let models = load_models(&state).await?;
        if !models.iter().any(|model| model.id == default_model_id) {
            return Err(AppError::NotFound("默认模型不存在".into()));
        }
        config.default_model_id = Some(default_model_id);
    }

    if let Some(temperature) = updates.temperature {
        config.temperature = temperature.clamp(0.0, 2.0);
    }

    if let Some(max_tokens) = updates.max_tokens {
        config.max_tokens = max_tokens.clamp(100, 100_000);
    }

    if let Some(timeout) = updates.timeout {
        config.timeout = timeout.clamp(5_000, 120_000);
    }

    if let Some(prompts) = updates.prompts {
        config.prompts = AiPrompts {
            explain: if prompts.explain.trim().is_empty() {
                DEFAULT_PROMPT_EXPLAIN.to_string()
            } else {
                prompts.explain
            },
            optimize: if prompts.optimize.trim().is_empty() {
                DEFAULT_PROMPT_OPTIMIZE.to_string()
            } else {
                prompts.optimize
            },
            write: if prompts.write.trim().is_empty() {
                DEFAULT_PROMPT_WRITE.to_string()
            } else {
                prompts.write
            },
        };
    }

    save_config(&state, &config).await
}

#[tauri::command]
pub async fn ai_request(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    action: AiAction,
    content: String,
    language: Option<String>,
    request_id: Option<String>,
) -> CmdResult<String> {
    state.auth.require_auth().await?;

    let config = load_config(&state).await?;
    let default_model_id = config
        .default_model_id
        .clone()
        .ok_or_else(|| AppError::Validation("请先设置默认 AI 模型".into()))?;

    perform_request_with_model(
        &state,
        &app_handle,
        action,
        content,
        default_model_id,
        language,
        request_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
    )
    .await
}

#[tauri::command]
pub async fn ai_request_with_model(
    state: State<'_, AppState>,
    app_handle: tauri::AppHandle,
    action: AiAction,
    content: String,
    model_id: String,
    language: Option<String>,
    request_id: Option<String>,
) -> CmdResult<String> {
    state.auth.require_auth().await?;
    perform_request_with_model(
        &state,
        &app_handle,
        action,
        content,
        model_id,
        language,
        request_id.unwrap_or_else(|| Uuid::new_v4().to_string()),
    )
    .await
}

#[tauri::command]
pub async fn ai_cancel_request(state: State<'_, AppState>, request_id: String) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let mut flags = state.ai_cancel_flags.lock().await;
    if let Some(flag) = flags.get(&request_id) {
        flag.store(true, Ordering::Relaxed);
    }
    flags.remove(&request_id);
    Ok(())
}

#[tauri::command]
pub async fn ai_get_chat_history(state: State<'_, AppState>) -> CmdResult<Vec<AiChatMessage>> {
    state.auth.require_auth().await?;
    load_chat_history(&state).await
}

#[tauri::command]
pub async fn ai_save_chat_history(
    state: State<'_, AppState>,
    messages: Vec<AiChatMessage>,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    save_chat_history(&state, &messages).await
}

#[tauri::command]
pub async fn ai_clear_chat_history(state: State<'_, AppState>) -> CmdResult<()> {
    state.auth.require_auth().await?;
    save_chat_history(&state, &[]).await
}

#[tauri::command]
pub async fn ai_get_terminal_chat_history(
    state: State<'_, AppState>,
    connection_id: String,
) -> CmdResult<Vec<AiChatMessage>> {
    state.auth.require_auth().await?;
    let mut history = load_terminal_chat_history(&state).await?;
    Ok(history.remove(&connection_id).unwrap_or_default())
}

#[tauri::command]
pub async fn ai_save_terminal_chat_history(
    state: State<'_, AppState>,
    connection_id: String,
    messages: Vec<AiChatMessage>,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let mut history = load_terminal_chat_history(&state).await?;
    history.insert(connection_id, messages);
    save_terminal_chat_history(&state, &history).await
}

#[tauri::command]
pub async fn ai_clear_terminal_chat_history(
    state: State<'_, AppState>,
    connection_id: String,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let mut history = load_terminal_chat_history(&state).await?;
    history.remove(&connection_id);
    save_terminal_chat_history(&state, &history).await
}

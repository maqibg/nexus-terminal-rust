//! 通知（Webhook/Email/Telegram）

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationChannel {
    pub id: i64,
    pub name: String,
    pub channel_type: String,
    pub config: String,
    pub enabled: bool,
}

/// 发送通知到指定渠道
pub async fn send_notification(
    channel: &NotificationChannel,
    title: &str,
    body: &str,
) -> Result<(), String> {
    let config: Value =
        serde_json::from_str(&channel.config).map_err(|e| format!("解析渠道配置失败: {e}"))?;

    match channel.channel_type.as_str() {
        "webhook" => send_webhook(&config, title, body).await,
        "email" => send_email(&config, title, body).await,
        "telegram" => send_telegram(&config, title, body).await,
        other => Err(format!("不支持的通知类型: {other}")),
    }
}

async fn send_webhook(config: &Value, title: &str, body: &str) -> Result<(), String> {
    let url = config["url"].as_str().ok_or("webhook 缺少 url 配置")?;
    let payload = serde_json::json!({ "title": title, "body": body });

    let client = reqwest::Client::new();
    let resp = client
        .post(url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Webhook 请求失败: {e}"))?;

    if !resp.status().is_success() {
        return Err(format!("Webhook 返回状态码: {}", resp.status()));
    }
    Ok(())
}

async fn send_email(_config: &Value, _title: &str, _body: &str) -> Result<(), String> {
    Err("Email 通知功能已禁用".to_string())
}

async fn send_telegram(config: &Value, title: &str, body: &str) -> Result<(), String> {
    let bot_token = config["bot_token"]
        .as_str()
        .ok_or("telegram 缺少 bot_token")?;
    let chat_id = config["chat_id"].as_str().ok_or("telegram 缺少 chat_id")?;

    let url = format!("https://api.telegram.org/bot{bot_token}/sendMessage");
    let text = format!("*{title}*\n{body}");
    let payload = serde_json::json!({
        "chat_id": chat_id,
        "text": text,
        "parse_mode": "Markdown",
    });

    let client = reqwest::Client::new();
    let resp = client
        .post(&url)
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("Telegram 请求失败: {e}"))?;

    if !resp.status().is_success() {
        let err_body = resp.text().await.unwrap_or_default();
        return Err(format!("Telegram 返回错误: {err_body}"));
    }
    Ok(())
}

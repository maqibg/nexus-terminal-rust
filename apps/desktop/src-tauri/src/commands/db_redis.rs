use api_contract::error::{AppError, CmdResult};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use redis::Value as RedisValue;
use serde::{Deserialize, Serialize};
use std::time::Instant;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
pub struct RedisRequestBase {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub encrypted_password: Option<String>,
    pub db: Option<u32>,
}

fn decrypt_password_if_needed(state: &AppState, base: &mut RedisRequestBase) -> Result<(), AppError> {
    if base
        .password
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .is_some()
    {
        return Ok(());
    }

    let encrypted = base
        .encrypted_password
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    if let Some(enc) = encrypted {
        base.password = Some(
            state
                .crypto
                .decrypt(enc)
                .map_err(|e| AppError::Crypto(e.to_string()))?,
        );
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
pub struct RedisScanKeysRequest {
    #[serde(flatten)]
    pub base: RedisRequestBase,
    pub pattern: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisKeyInfo {
    pub key: String,
    pub key_type: String,
}

#[derive(Debug, Deserialize)]
pub struct RedisKeyDetailRequest {
    #[serde(flatten)]
    pub base: RedisRequestBase,
    pub key: String,
    pub preview_limit: Option<u32>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisKeyDetail {
    pub key: String,
    pub key_type: String,
    pub ttl_seconds: Option<i64>,
    pub encoding: Option<String>,
    pub memory_usage_bytes: Option<i64>,
    pub length: Option<i64>,
    pub meta_error: Option<String>,
    pub value: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub struct RedisCommandRequest {
    #[serde(flatten)]
    pub base: RedisRequestBase,
    pub args: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RedisCommandResult {
    pub result: serde_json::Value,
    pub duration_ms: u64,
}

fn redis_url(base: &RedisRequestBase) -> Result<String, AppError> {
    let host = base.host.trim();
    if host.is_empty() {
        return Err(AppError::Validation("host is required".into()));
    }

    let db = base.db.unwrap_or(0);
    let mut url = format!("redis://{host}:{}{}", base.port, format!("/{db}"));

    let username = base
        .username
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());
    let password = base
        .password
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty());

    if username.is_some() || password.is_some() {
        let auth = match (username, password) {
            (Some(user), Some(pw)) => format!("{user}:{pw}"),
            (None, Some(pw)) => format!(":{pw}"),
            (Some(user), None) => user.to_string(),
            (None, None) => String::new(),
        };
        url = format!("redis://{auth}@{host}:{}{}", base.port, format!("/{db}"));
    }

    Ok(url)
}

async fn open_redis_connection(
    base: &RedisRequestBase,
) -> Result<redis::aio::MultiplexedConnection, AppError> {
    let url = redis_url(base)?;
    let client = redis::Client::open(url).map_err(|e| AppError::Database(e.to_string()))?;
    client
        .get_multiplexed_async_connection()
        .await
        .map_err(|e| AppError::Database(e.to_string()))
}

fn bytes_to_json(bytes: &[u8]) -> serde_json::Value {
    match std::str::from_utf8(bytes) {
        Ok(s) => serde_json::Value::String(s.to_string()),
        Err(_) => serde_json::Value::String(format!("base64:{}", STANDARD.encode(bytes))),
    }
}

fn redis_value_to_json(value: &RedisValue) -> serde_json::Value {
    use serde_json::{Map, Number, Value};

    match value {
        RedisValue::Nil => Value::Null,
        RedisValue::Int(n) => Value::Number((*n).into()),
        RedisValue::BulkString(bytes) => bytes_to_json(bytes),
        RedisValue::Array(items) => Value::Array(items.iter().map(redis_value_to_json).collect()),
        RedisValue::SimpleString(text) => Value::String(text.to_string()),
        RedisValue::Okay => Value::String("OK".into()),
        RedisValue::Map(items) => Value::Array(
            items
                .iter()
                .map(|(k, v)| Value::Array(vec![redis_value_to_json(k), redis_value_to_json(v)]))
                .collect(),
        ),
        RedisValue::Attribute { data, attributes } => {
            let attrs = Value::Array(
                attributes
                    .iter()
                    .map(|(k, v)| Value::Array(vec![redis_value_to_json(k), redis_value_to_json(v)]))
                    .collect(),
            );
            let mut obj = Map::new();
            obj.insert("data".into(), redis_value_to_json(data));
            obj.insert("attributes".into(), attrs);
            Value::Object(obj)
        }
        RedisValue::Set(items) => Value::Array(items.iter().map(redis_value_to_json).collect()),
        RedisValue::Double(n) => Number::from_f64(*n).map(Value::Number).unwrap_or(Value::Null),
        RedisValue::Boolean(b) => Value::Bool(*b),
        RedisValue::VerbatimString { format, text } => {
            let mut obj = Map::new();
            obj.insert("format".into(), Value::String(format!("{format:?}")));
            obj.insert("text".into(), Value::String(text.to_string()));
            Value::Object(obj)
        }
        RedisValue::BigNumber(n) => Value::String(n.to_string()),
        RedisValue::Push { kind, data } => {
            let mut obj = Map::new();
            obj.insert("kind".into(), Value::String(format!("{kind:?}")));
            obj.insert(
                "data".into(),
                Value::Array(data.iter().map(redis_value_to_json).collect()),
            );
            Value::Object(obj)
        }
        RedisValue::ServerError(err) => Value::String(format!("{err:?}")),
    }
}

fn normalize_scan_pattern(pattern: Option<String>) -> Option<String> {
    pattern
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

fn clamp_limit(limit: Option<u32>, default_value: u32, hard_max: u32) -> usize {
    limit
        .unwrap_or(default_value)
        .max(1)
        .min(hard_max) as usize
}

#[tauri::command]
pub async fn db_redis_scan_keys(
    state: State<'_, AppState>,
    mut req: RedisScanKeysRequest,
) -> CmdResult<Vec<RedisKeyInfo>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let pattern = normalize_scan_pattern(req.pattern);
    let limit = clamp_limit(req.limit, 200, 2000);
    let mut conn = open_redis_connection(&req.base).await?;

    let mut cursor: u64 = 0;
    let mut keys: Vec<String> = Vec::new();
    while keys.len() < limit {
        let batch = (limit - keys.len()).min(250);
        let mut cmd = redis::cmd("SCAN");
        cmd.arg(cursor);
        if let Some(pat) = pattern.as_deref() {
            cmd.arg("MATCH").arg(pat);
        }
        cmd.arg("COUNT").arg(batch);

        let (next_cursor, mut batch_keys): (u64, Vec<String>) = cmd
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;

        keys.append(&mut batch_keys);
        cursor = next_cursor;
        if cursor == 0 {
            break;
        }
    }
    keys.truncate(limit);

    let mut types: Vec<String> = Vec::new();
    if !keys.is_empty() {
        let mut pipe = redis::pipe();
        for key in &keys {
            pipe.cmd("TYPE").arg(key);
        }
        types = pipe
            .query_async(&mut conn)
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    Ok(keys
        .into_iter()
        .enumerate()
        .map(|(idx, key)| RedisKeyInfo {
            key,
            key_type: types.get(idx).cloned().unwrap_or_else(|| "unknown".into()),
        })
        .collect())
}

async fn redis_key_ttl_seconds(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
) -> Result<Option<i64>, AppError> {
    let ttl: i64 = redis::cmd("TTL")
        .arg(key)
        .query_async(conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if ttl >= 0 {
        Ok(Some(ttl))
    } else {
        Ok(None)
    }
}

fn redis_preview_limit(limit: Option<u32>) -> usize {
    clamp_limit(limit, 200, 2000)
}

async fn redis_key_value_by_type(
    conn: &mut redis::aio::MultiplexedConnection,
    key: &str,
    key_type: &str,
    preview_limit: usize,
) -> Result<serde_json::Value, AppError> {
    use serde_json::Value;

    match key_type {
        "string" => {
            let raw: Option<Vec<u8>> = redis::cmd("GET")
                .arg(key)
                .query_async(conn)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;
            Ok(raw.map(|b| bytes_to_json(&b)).unwrap_or(Value::Null))
        }
        "hash" => {
            let pairs: Vec<(Vec<u8>, Vec<u8>)> = redis::cmd("HGETALL")
                .arg(key)
                .query_async(conn)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(Value::Array(
                pairs
                    .into_iter()
                    .take(preview_limit)
                    .map(|(field, value)| {
                        Value::Array(vec![bytes_to_json(&field), bytes_to_json(&value)])
                    })
                    .collect(),
            ))
        }
        "list" => {
            let items: Vec<Vec<u8>> = redis::cmd("LRANGE")
                .arg(key)
                .arg(0i64)
                .arg((preview_limit.saturating_sub(1)) as i64)
                .query_async(conn)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(Value::Array(items.into_iter().map(|v| bytes_to_json(&v)).collect()))
        }
        "set" => {
            let members: Vec<Vec<u8>> = redis::cmd("SMEMBERS")
                .arg(key)
                .query_async(conn)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(Value::Array(
                members
                    .into_iter()
                    .take(preview_limit)
                    .map(|v| bytes_to_json(&v))
                    .collect(),
            ))
        }
        "zset" => {
            let pairs: Vec<(Vec<u8>, f64)> = redis::cmd("ZRANGE")
                .arg(key)
                .arg(0i64)
                .arg((preview_limit.saturating_sub(1)) as i64)
                .arg("WITHSCORES")
                .query_async(conn)
                .await
                .map_err(|e| AppError::Database(e.to_string()))?;

            Ok(Value::Array(
                pairs
                    .into_iter()
                    .map(|(member, score)| {
                        Value::Array(vec![
                            bytes_to_json(&member),
                            Value::Number(
                                serde_json::Number::from_f64(score).unwrap_or_else(|| 0.into()),
                            ),
                        ])
                    })
                    .collect(),
            ))
        }
        "none" => Ok(Value::Null),
        _ => Ok(Value::String(format!("<unsupported type: {key_type}>"))),
    }
}

#[tauri::command]
pub async fn db_redis_get_key(
    state: State<'_, AppState>,
    mut req: RedisKeyDetailRequest,
) -> CmdResult<RedisKeyDetail> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let key = req.key.trim();
    if key.is_empty() {
        return Err(AppError::Validation("key is required".into()));
    }

    let mut conn = open_redis_connection(&req.base).await?;
    let key_type: String = redis::cmd("TYPE")
        .arg(key)
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let ttl_seconds = redis_key_ttl_seconds(&mut conn, key).await?;
    let preview_limit = redis_preview_limit(req.preview_limit);
    let value = redis_key_value_by_type(&mut conn, key, &key_type, preview_limit).await?;

    let mut meta_errors: Vec<String> = Vec::new();

    let encoding: Option<String> = match redis::cmd("OBJECT")
        .arg("ENCODING")
        .arg(key)
        .query_async(&mut conn)
        .await
    {
        Ok(v) => Some(v),
        Err(e) => {
            meta_errors.push(format!("OBJECT ENCODING failed: {e}"));
            None
        }
    };

    let memory_usage_bytes: Option<i64> = match redis::cmd("MEMORY")
        .arg("USAGE")
        .arg(key)
        .query_async(&mut conn)
        .await
    {
        Ok(v) => Some(v),
        Err(e) => {
            meta_errors.push(format!("MEMORY USAGE failed: {e}"));
            None
        }
    };

    let length: Option<i64> = match key_type.as_str() {
        "string" => redis::cmd("STRLEN").arg(key).query_async(&mut conn).await.ok(),
        "hash" => redis::cmd("HLEN").arg(key).query_async(&mut conn).await.ok(),
        "list" => redis::cmd("LLEN").arg(key).query_async(&mut conn).await.ok(),
        "set" => redis::cmd("SCARD").arg(key).query_async(&mut conn).await.ok(),
        "zset" => redis::cmd("ZCARD").arg(key).query_async(&mut conn).await.ok(),
        _ => None,
    };

    if length.is_none() && matches!(key_type.as_str(), "string" | "hash" | "list" | "set" | "zset") {
        meta_errors.push("length lookup failed".into());
    }

    Ok(RedisKeyDetail {
        key: key.to_string(),
        key_type,
        ttl_seconds,
        encoding,
        memory_usage_bytes,
        length,
        meta_error: if meta_errors.is_empty() {
            None
        } else {
            Some(meta_errors.join("\n"))
        },
        value,
    })
}

#[tauri::command]
pub async fn db_redis_command(
    state: State<'_, AppState>,
    mut req: RedisCommandRequest,
) -> CmdResult<RedisCommandResult> {
    state.auth.require_auth().await?;

    if req.args.is_empty() {
        return Err(AppError::Validation("args is required".into()));
    }

    decrypt_password_if_needed(&state, &mut req.base)?;
    let mut conn = open_redis_connection(&req.base).await?;

    let started = Instant::now();
    let mut cmd = redis::cmd(&req.args[0]);
    for arg in &req.args[1..] {
        cmd.arg(arg);
    }

    let value: RedisValue = cmd
        .query_async(&mut conn)
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    Ok(RedisCommandResult {
        result: redis_value_to_json(&value),
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

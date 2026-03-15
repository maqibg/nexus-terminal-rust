use api_contract::error::{AppError, CmdResult};
use serde::Deserialize;
use std::time::Instant;
use tauri::State;

use crate::state::AppState;

use super::db_types::{DbColumn, DbQueryResult, DbTable};

#[derive(Debug, Deserialize)]
pub struct ClickHouseRequestBase {
    pub host: String,
    pub port: u16,
    pub username: Option<String>,
    pub password: Option<String>,
    pub encrypted_password: Option<String>,
    pub database: String,
    pub https: Option<bool>,
}

fn decrypt_password_if_needed(
    state: &AppState,
    base: &mut ClickHouseRequestBase,
) -> Result<(), AppError> {
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
pub struct ClickHouseListColumnsRequest {
    #[serde(flatten)]
    pub base: ClickHouseRequestBase,
    pub table_name: String,
}

#[derive(Debug, Deserialize)]
pub struct ClickHouseQueryRequest {
    #[serde(flatten)]
    pub base: ClickHouseRequestBase,
    pub sql: String,
}

fn require_non_empty(label: &str, input: &str) -> Result<(), AppError> {
    if input.trim().is_empty() {
        return Err(AppError::Validation(format!("{label} is required")));
    }
    Ok(())
}

fn validate_simple_name(label: &str, input: &str) -> Result<String, AppError> {
    let trimmed = input.trim();
    require_non_empty(label, trimmed)?;
    if !trimmed
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AppError::Validation(format!(
            "{label} contains unsupported characters"
        )));
    }
    Ok(trimmed.to_string())
}

fn clickhouse_base_url(base: &ClickHouseRequestBase) -> Result<String, AppError> {
    let host = base.host.trim();
    require_non_empty("host", host)?;
    let scheme = if base.https.unwrap_or(false) {
        "https"
    } else {
        "http"
    };
    Ok(format!("{scheme}://{host}:{}", base.port))
}

fn has_format_clause(sql: &str) -> bool {
    sql.to_ascii_lowercase().contains("format")
}

async fn clickhouse_post(
    base: &ClickHouseRequestBase,
    sql: &str,
) -> Result<String, AppError> {
    let url = clickhouse_base_url(base)?;
    let database = validate_simple_name("database", &base.database)?;

    let client = reqwest::Client::new();
    let mut req = client
        .post(format!("{url}/"))
        .query(&[("database", database.as_str()), ("default_format", "JSONCompact")])
        .body(sql.to_string());

    if let Some(user) = base.username.as_deref().map(str::trim).filter(|s| !s.is_empty()) {
        req = req.basic_auth(user, base.password.clone());
    }

    let resp = req
        .send()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    let status = resp.status();
    let text = resp
        .text()
        .await
        .map_err(|e| AppError::Database(e.to_string()))?;

    if !status.is_success() {
        return Err(AppError::Database(format!(
            "clickhouse http {}: {}",
            status.as_u16(),
            text.trim()
        )));
    }

    Ok(text)
}

fn parse_jsoncompact(text: &str) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>), AppError> {
    let value: serde_json::Value = serde_json::from_str(text)
        .map_err(|e| AppError::Database(format!("clickhouse json parse failed: {e}")))?;

    let meta = value
        .get("meta")
        .and_then(|v| v.as_array())
        .ok_or_else(|| AppError::Database("clickhouse response missing meta".into()))?;

    let columns = meta
        .iter()
        .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
        .collect::<Vec<_>>();

    let mut rows = Vec::new();
    if let Some(data) = value.get("data").and_then(|v| v.as_array()) {
        rows = Vec::with_capacity(data.len());
        for row in data {
        let arr = row
            .as_array()
            .ok_or_else(|| AppError::Database("clickhouse row is not array".into()))?;
        rows.push(arr.iter().cloned().collect());
        }
    }

    Ok((columns, rows))
}

#[tauri::command]
pub async fn db_clickhouse_list_tables(
    state: State<'_, AppState>,
    mut req: ClickHouseRequestBase,
) -> CmdResult<Vec<DbTable>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req)?;
    let sql = "SELECT name, if(lower(engine) LIKE '%view%', 'view', 'table') AS kind \
               FROM system.tables \
               WHERE database = currentDatabase() \
               ORDER BY kind, name \
               FORMAT JSONCompact";

    let text = clickhouse_post(&req, sql).await?;
    let (_cols, rows) = parse_jsoncompact(&text)?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let name = row
            .get(0)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let kind = row
            .get(1)
            .and_then(|v| v.as_str())
            .unwrap_or("table")
            .to_string();
        if !name.is_empty() {
            items.push(DbTable { name, kind });
        }
    }
    Ok(items)
}

#[tauri::command]
pub async fn db_clickhouse_list_columns(
    state: State<'_, AppState>,
    mut req: ClickHouseListColumnsRequest,
) -> CmdResult<Vec<DbColumn>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let table = validate_simple_name("table_name", &req.table_name)?;
    let sql = format!(
        "SELECT position AS cid, name, type AS data_type, \
         if(startsWith(type, 'Nullable('), 0, 1) AS not_null, \
         default_expression AS default_value, is_in_primary_key AS primary_key \
         FROM system.columns \
         WHERE database = currentDatabase() AND table = '{table}' \
         ORDER BY position \
         FORMAT JSONCompact"
    );

    let text = clickhouse_post(&req.base, &sql).await?;
    let (_cols, rows) = parse_jsoncompact(&text)?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let cid = row
            .get(0)
            .and_then(|v| v.as_i64())
            .unwrap_or_default() as i32;
        let name = row
            .get(1)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let data_type = row
            .get(2)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let not_null_raw = row.get(3).and_then(|v| v.as_i64()).unwrap_or(0);
        let default_value = row
            .get(4)
            .and_then(|v| v.as_str())
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string);
        let primary_key_raw = row.get(5).and_then(|v| v.as_i64()).unwrap_or(0);

        if !name.is_empty() {
            items.push(DbColumn {
                cid,
                name,
                data_type,
                not_null: not_null_raw != 0,
                default_value,
                primary_key: primary_key_raw != 0,
            });
        }
    }

    Ok(items)
}

#[tauri::command]
pub async fn db_clickhouse_query(
    state: State<'_, AppState>,
    mut req: ClickHouseQueryRequest,
) -> CmdResult<DbQueryResult> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let sql = req.sql.trim();
    require_non_empty("sql", sql)?;

    let sql_to_send = if has_format_clause(sql) {
        sql.to_string()
    } else {
        format!("{sql}\nFORMAT JSONCompact")
    };

    let started = Instant::now();
    let text = clickhouse_post(&req.base, &sql_to_send).await?;
    match parse_jsoncompact(&text) {
        Ok((columns, rows)) => Ok(DbQueryResult {
            columns,
            rows,
            rows_affected: None,
            duration_ms: started.elapsed().as_millis() as u64,
        }),
        Err(_) => Ok(DbQueryResult {
            columns: vec!["output".into()],
            rows: vec![vec![serde_json::Value::String(text)]],
            rows_affected: None,
            duration_ms: started.elapsed().as_millis() as u64,
        }),
    }
}

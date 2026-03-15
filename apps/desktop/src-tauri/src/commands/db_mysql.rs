use api_contract::error::{AppError, CmdResult};
use base64::Engine as _;
use serde::Deserialize;
use sqlx::{mysql::MySqlConnectOptions, mysql::MySqlPoolOptions, Column, Executor, Row};
use std::time::Instant;
use tauri::State;

use crate::state::AppState;

use super::db_types::{DbColumn, DbQueryResult, DbTable};

#[derive(Debug, Deserialize)]
pub struct MysqlRequestBase {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub encrypted_password: Option<String>,
    pub database: String,
}

#[derive(Debug, Deserialize)]
pub struct MysqlListColumnsRequest {
    #[serde(flatten)]
    pub base: MysqlRequestBase,
    pub table_name: String,
}

#[derive(Debug, Deserialize)]
pub struct MysqlQueryRequest {
    #[serde(flatten)]
    pub base: MysqlRequestBase,
    pub sql: String,
}

async fn open_mysql_pool(base: &MysqlRequestBase) -> Result<sqlx::MySqlPool, AppError> {
    let host = base.host.trim();
    let username = base.username.trim();
    let database = base.database.trim();
    if host.is_empty() {
        return Err(AppError::Validation("host is required".into()));
    }
    if username.is_empty() {
        return Err(AppError::Validation("username is required".into()));
    }
    if database.is_empty() {
        return Err(AppError::Validation("database is required".into()));
    }

    let mut options = MySqlConnectOptions::new()
        .host(host)
        .port(base.port)
        .username(username)
        .database(database);
    if let Some(pw) = base.password.as_deref() {
        options = options.password(pw);
    }

    MySqlPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| AppError::Database(error.to_string()))
}

fn decrypt_password_if_needed(state: &AppState, base: &mut MysqlRequestBase) -> Result<(), AppError> {
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

fn mysql_table_kind(raw: &str) -> String {
    if raw.eq_ignore_ascii_case("VIEW") {
        "view".into()
    } else {
        "table".into()
    }
}

fn is_read_query(sql: &str) -> bool {
    let trimmed = sql.trim_start();
    if trimmed.is_empty() {
        return false;
    }
    let head = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        head.as_str(),
        "select" | "with" | "show" | "desc" | "describe" | "explain"
    )
}

fn mysql_cell_to_json(row: &sqlx::mysql::MySqlRow, idx: usize) -> serde_json::Value {
    use serde_json::Value;

    if let Ok(value) = row.try_get::<Option<i64>, _>(idx) {
        return value.map(Value::from).unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<f64>, _>(idx) {
        return value
            .and_then(serde_json::Number::from_f64)
            .map(Value::Number)
            .unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<bool>, _>(idx) {
        return value.map(Value::from).unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<String>, _>(idx) {
        return value.map(Value::from).unwrap_or(Value::Null);
    }
    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(idx) {
        if let Some(bytes) = value {
            return Value::String(format!("base64:{}", base64::engine::general_purpose::STANDARD.encode(bytes)));
        }
        return Value::Null;
    }

    Value::String("<unsupported>".into())
}

async fn mysql_describe_columns(pool: &sqlx::MySqlPool, sql: &str) -> Result<Vec<String>, AppError> {
    let describe = pool
        .describe(sql)
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;
    Ok(describe
        .columns()
        .iter()
        .map(|c| c.name().to_string())
        .collect::<Vec<_>>())
}

async fn mysql_fetch_rows(
    pool: &sqlx::MySqlPool,
    sql: &str,
    columns_len: usize,
) -> Result<Vec<Vec<serde_json::Value>>, AppError> {
    let rows = sqlx::query(sql)
        .fetch_all(pool)
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    let mut out_rows = Vec::with_capacity(rows.len());
    for row in rows {
        let mut out_row = Vec::with_capacity(columns_len);
        for idx in 0..columns_len {
            out_row.push(mysql_cell_to_json(&row, idx));
        }
        out_rows.push(out_row);
    }
    Ok(out_rows)
}

async fn mysql_run_read_query(
    pool: &sqlx::MySqlPool,
    sql: &str,
    started: Instant,
) -> Result<DbQueryResult, AppError> {
    let columns = mysql_describe_columns(pool, sql).await?;
    let rows = mysql_fetch_rows(pool, sql, columns.len()).await?;
    Ok(DbQueryResult {
        columns,
        rows,
        rows_affected: None,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

async fn mysql_run_exec(
    pool: &sqlx::MySqlPool,
    sql: &str,
    started: Instant,
) -> Result<DbQueryResult, AppError> {
    let result = sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;
    Ok(DbQueryResult {
        columns: vec![],
        rows: vec![],
        rows_affected: Some(result.rows_affected()),
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn db_mysql_list_tables(
    state: State<'_, AppState>,
    mut req: MysqlRequestBase,
) -> CmdResult<Vec<DbTable>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req)?;
    let pool = open_mysql_pool(&req).await?;
    let rows = sqlx::query(
        "SELECT table_name AS name, table_type AS kind \
         FROM information_schema.tables \
         WHERE table_schema = ? AND table_type IN ('BASE TABLE', 'VIEW') \
         ORDER BY table_type, table_name",
    )
    .bind(req.database.trim())
    .fetch_all(&pool)
    .await
    .map_err(|error| AppError::Database(error.to_string()))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row
            .try_get("name")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let raw_kind: String = row
            .try_get("kind")
            .map_err(|e| AppError::Database(e.to_string()))?;
        items.push(DbTable {
            name,
            kind: mysql_table_kind(&raw_kind),
        });
    }
    Ok(items)
}

#[tauri::command]
pub async fn db_mysql_list_columns(
    state: State<'_, AppState>,
    mut req: MysqlListColumnsRequest,
) -> CmdResult<Vec<DbColumn>> {
    state.auth.require_auth().await?;

    let table = req.table_name.trim();
    if table.is_empty() {
        return Err(AppError::Validation("table_name is required".into()));
    }

    decrypt_password_if_needed(&state, &mut req.base)?;
    let pool = open_mysql_pool(&req.base).await?;
    let rows = sqlx::query(
        "SELECT ordinal_position AS cid, column_name AS name, column_type AS data_type, \
         is_nullable AS is_nullable, column_default AS default_value, column_key AS column_key \
         FROM information_schema.columns \
         WHERE table_schema = ? AND table_name = ? \
         ORDER BY ordinal_position",
    )
    .bind(req.base.database.trim())
    .bind(table)
    .fetch_all(&pool)
    .await
    .map_err(|error| AppError::Database(error.to_string()))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let cid: i32 = row
            .try_get("cid")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let name: String = row
            .try_get("name")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let data_type: String = row
            .try_get("data_type")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let nullable: String = row
            .try_get("is_nullable")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let default_value: Option<String> = row
            .try_get("default_value")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let column_key: Option<String> = row
            .try_get("column_key")
            .map_err(|e| AppError::Database(e.to_string()))?;

        items.push(DbColumn {
            cid,
            name,
            data_type,
            not_null: nullable.eq_ignore_ascii_case("NO"),
            default_value,
            primary_key: column_key
                .as_deref()
                .map(|k| k.eq_ignore_ascii_case("PRI"))
                .unwrap_or(false),
        });
    }

    Ok(items)
}

#[tauri::command]
pub async fn db_mysql_query(
    state: State<'_, AppState>,
    mut req: MysqlQueryRequest,
) -> CmdResult<DbQueryResult> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let sql = req.sql.trim();
    if sql.is_empty() {
        return Err(AppError::Validation("sql is required".into()));
    }

    let pool = open_mysql_pool(&req.base).await?;
    let started = Instant::now();
    if is_read_query(sql) {
        return mysql_run_read_query(&pool, sql, started).await;
    }
    mysql_run_exec(&pool, sql, started).await
}

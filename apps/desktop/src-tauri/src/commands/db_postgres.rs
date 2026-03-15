use api_contract::error::{AppError, CmdResult};
use base64::Engine as _;
use serde::Deserialize;
use sqlx::{postgres::PgConnectOptions, postgres::PgPoolOptions, Column, Executor, Row};
use std::time::Instant;
use tauri::State;

use crate::state::AppState;

use super::db_types::{DbColumn, DbQueryResult, DbTable};

#[derive(Debug, Deserialize)]
pub struct PostgresRequestBase {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub encrypted_password: Option<String>,
    pub database: String,
    pub schema: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PostgresListColumnsRequest {
    #[serde(flatten)]
    pub base: PostgresRequestBase,
    pub table_name: String,
}

#[derive(Debug, Deserialize)]
pub struct PostgresQueryRequest {
    #[serde(flatten)]
    pub base: PostgresRequestBase,
    pub sql: String,
}

async fn open_pg_pool(base: &PostgresRequestBase) -> Result<sqlx::PgPool, AppError> {
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

    let mut options = PgConnectOptions::new()
        .host(host)
        .port(base.port)
        .username(username)
        .database(database);
    if let Some(pw) = base.password.as_deref() {
        options = options.password(pw);
    }

    PgPoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| AppError::Database(error.to_string()))
}

fn decrypt_password_if_needed(
    state: &AppState,
    base: &mut PostgresRequestBase,
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

fn pg_table_kind(raw: &str) -> String {
    if raw.eq_ignore_ascii_case("VIEW") {
        "view".into()
    } else {
        "table".into()
    }
}

fn pg_schema(base: &PostgresRequestBase) -> String {
    base.schema
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("public")
        .to_string()
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

fn pg_cell_to_json(row: &sqlx::postgres::PgRow, idx: usize) -> serde_json::Value {
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
    if let Ok(value) = row.try_get::<Option<uuid::Uuid>, _>(idx) {
        return value.map(|v| Value::String(v.to_string())).unwrap_or(Value::Null);
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

async fn pg_describe_columns(pool: &sqlx::PgPool, sql: &str) -> Result<Vec<String>, AppError> {
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

async fn pg_fetch_rows(
    pool: &sqlx::PgPool,
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
            out_row.push(pg_cell_to_json(&row, idx));
        }
        out_rows.push(out_row);
    }
    Ok(out_rows)
}

async fn pg_run_read_query(
    pool: &sqlx::PgPool,
    sql: &str,
    started: Instant,
) -> Result<DbQueryResult, AppError> {
    let columns = pg_describe_columns(pool, sql).await?;
    let rows = pg_fetch_rows(pool, sql, columns.len()).await?;
    Ok(DbQueryResult {
        columns,
        rows,
        rows_affected: None,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

async fn pg_run_exec(
    pool: &sqlx::PgPool,
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
pub async fn db_postgres_list_tables(
    state: State<'_, AppState>,
    mut req: PostgresRequestBase,
) -> CmdResult<Vec<DbTable>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req)?;
    let schema = pg_schema(&req);
    let pool = open_pg_pool(&req).await?;
    let rows = sqlx::query(
        "SELECT table_name AS name, table_type AS kind \
         FROM information_schema.tables \
         WHERE table_schema = $1 AND table_type IN ('BASE TABLE', 'VIEW') \
         ORDER BY table_type, table_name",
    )
    .bind(schema)
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
            kind: pg_table_kind(&raw_kind),
        });
    }
    Ok(items)
}

#[tauri::command]
pub async fn db_postgres_list_columns(
    state: State<'_, AppState>,
    mut req: PostgresListColumnsRequest,
) -> CmdResult<Vec<DbColumn>> {
    state.auth.require_auth().await?;

    let schema = pg_schema(&req.base);
    let table = req.table_name.trim();
    if table.is_empty() {
        return Err(AppError::Validation("table_name is required".into()));
    }

    decrypt_password_if_needed(&state, &mut req.base)?;
    let pool = open_pg_pool(&req.base).await?;
    let rows = sqlx::query(
        "SELECT c.ordinal_position AS cid, c.column_name AS name, c.data_type AS data_type, \
         c.is_nullable AS is_nullable, c.column_default AS default_value, \
         CASE WHEN tc.constraint_type = 'PRIMARY KEY' THEN true ELSE false END AS primary_key \
         FROM information_schema.columns c \
         LEFT JOIN information_schema.key_column_usage kcu \
           ON kcu.table_schema=c.table_schema AND kcu.table_name=c.table_name AND kcu.column_name=c.column_name \
         LEFT JOIN information_schema.table_constraints tc \
           ON tc.constraint_name=kcu.constraint_name AND tc.table_schema=kcu.table_schema AND tc.table_name=kcu.table_name \
         WHERE c.table_schema=$1 AND c.table_name=$2 \
         ORDER BY c.ordinal_position",
    )
    .bind(schema)
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
        let primary_key: bool = row
            .try_get("primary_key")
            .map_err(|e| AppError::Database(e.to_string()))?;

        items.push(DbColumn {
            cid,
            name,
            data_type,
            not_null: nullable.eq_ignore_ascii_case("NO"),
            default_value,
            primary_key,
        });
    }
    Ok(items)
}

#[tauri::command]
pub async fn db_postgres_query(
    state: State<'_, AppState>,
    mut req: PostgresQueryRequest,
) -> CmdResult<DbQueryResult> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let sql = req.sql.trim();
    if sql.is_empty() {
        return Err(AppError::Validation("sql is required".into()));
    }

    let pool = open_pg_pool(&req.base).await?;
    let started = Instant::now();
    if is_read_query(sql) {
        return pg_run_read_query(&pool, sql, started).await;
    }
    pg_run_exec(&pool, sql, started).await
}

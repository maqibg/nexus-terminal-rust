use api_contract::error::AppError;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use sqlx::{sqlite::SqliteConnectOptions, sqlite::SqlitePoolOptions, Column, Executor, Row};
use std::path::PathBuf;
use std::time::Instant;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteTable {
    pub name: String,
    pub kind: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteColumn {
    pub cid: i32,
    pub name: String,
    pub data_type: String,
    pub not_null: bool,
    pub default_value: Option<String>,
    pub primary_key: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SqliteQueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub rows_affected: Option<u64>,
    pub duration_ms: u64,
}

async fn open_sqlite_pool(db_path: &str) -> Result<sqlx::SqlitePool, AppError> {
    let path = PathBuf::from(db_path);
    if !path.exists() {
        return Err(AppError::Validation(format!(
            "sqlite file not found: {db_path}"
        )));
    }

    let options = SqliteConnectOptions::new()
        .filename(path)
        .create_if_missing(false);

    SqlitePoolOptions::new()
        .max_connections(1)
        .connect_with(options)
        .await
        .map_err(|error| AppError::Database(error.to_string()))
}

fn sqlite_quote_identifier(input: &str) -> String {
    format!("\"{}\"", input.replace('\"', "\"\""))
}

fn is_sqlite_query(sql: &str) -> bool {
    let trimmed = sql.trim_start();
    if trimmed.is_empty() {
        return false;
    }

    let head = trimmed
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();

    matches!(head.as_str(), "select" | "with" | "pragma" | "explain")
}

fn sqlite_cell_to_json(row: &sqlx::sqlite::SqliteRow, idx: usize) -> serde_json::Value {
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

    if let Ok(value) = row.try_get::<Option<String>, _>(idx) {
        return value.map(Value::from).unwrap_or(Value::Null);
    }

    if let Ok(value) = row.try_get::<Option<Vec<u8>>, _>(idx) {
        if let Some(bytes) = value {
            return Value::String(format!("base64:{}", STANDARD.encode(bytes)));
        }
        return Value::Null;
    }

    Value::String("<unsupported>".into())
}

async fn sqlite_describe_columns(
    pool: &sqlx::SqlitePool,
    sql: &str,
) -> Result<Vec<String>, AppError> {
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

async fn sqlite_fetch_rows(
    pool: &sqlx::SqlitePool,
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
            out_row.push(sqlite_cell_to_json(&row, idx));
        }
        out_rows.push(out_row);
    }
    Ok(out_rows)
}

async fn sqlite_run_read_query(
    pool: &sqlx::SqlitePool,
    sql: &str,
    started: Instant,
) -> Result<SqliteQueryResult, AppError> {
    let columns = sqlite_describe_columns(pool, sql).await?;
    let rows = sqlite_fetch_rows(pool, sql, columns.len()).await?;

    Ok(SqliteQueryResult {
        columns,
        rows,
        rows_affected: None,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

async fn sqlite_run_exec(
    pool: &sqlx::SqlitePool,
    sql: &str,
    started: Instant,
) -> Result<SqliteQueryResult, AppError> {
    let result = sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    Ok(SqliteQueryResult {
        columns: vec![],
        rows: vec![],
        rows_affected: Some(result.rows_affected()),
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn db_sqlite_list_tables(
    state: State<'_, AppState>,
    db_path: String,
) -> Result<Vec<SqliteTable>, AppError> {
    state.auth.require_auth().await?;

    let pool = open_sqlite_pool(&db_path).await?;
    let rows = sqlx::query(
        "SELECT name, type FROM sqlite_master \
         WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' \
         ORDER BY type, name",
    )
    .fetch_all(&pool)
    .await
    .map_err(|error| AppError::Database(error.to_string()))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row
            .try_get("name")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let kind: String = row
            .try_get("type")
            .map_err(|e| AppError::Database(e.to_string()))?;
        items.push(SqliteTable { name, kind });
    }

    Ok(items)
}

#[tauri::command]
pub async fn db_sqlite_list_columns(
    state: State<'_, AppState>,
    db_path: String,
    table_name: String,
) -> Result<Vec<SqliteColumn>, AppError> {
    state.auth.require_auth().await?;

    let pool = open_sqlite_pool(&db_path).await?;
    let sql = format!(
        "PRAGMA table_info({})",
        sqlite_quote_identifier(&table_name)
    );
    let rows = sqlx::query(&sql)
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
            .try_get("type")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let not_null_raw: i64 = row
            .try_get("notnull")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let default_value: Option<String> = row
            .try_get("dflt_value")
            .map_err(|e| AppError::Database(e.to_string()))?;
        let pk_raw: i64 = row
            .try_get("pk")
            .map_err(|e| AppError::Database(e.to_string()))?;

        items.push(SqliteColumn {
            cid,
            name,
            data_type,
            not_null: not_null_raw != 0,
            default_value,
            primary_key: pk_raw != 0,
        });
    }

    Ok(items)
}

#[tauri::command]
pub async fn db_sqlite_query(
    state: State<'_, AppState>,
    db_path: String,
    sql: String,
) -> Result<SqliteQueryResult, AppError> {
    state.auth.require_auth().await?;

    let sql = sql.trim();
    if sql.is_empty() {
        return Err(AppError::Validation("sql is required".into()));
    }

    let pool = open_sqlite_pool(&db_path).await?;

    let started = Instant::now();
    if is_sqlite_query(&sql) {
        return sqlite_run_read_query(&pool, sql, started).await;
    }

    sqlite_run_exec(&pool, sql, started).await
}

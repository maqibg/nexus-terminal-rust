use api_contract::error::{AppError, CmdResult};
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

use super::db_types::{DbColumn, DbQueryResult, DbTable};

#[cfg(windows)]
use std::time::Instant;

#[derive(Debug, Deserialize)]
pub struct OracleRequestBase {
    /// ODBC 连接串（建议不包含密码，例如：`DSN=MyOracle;` 或 `Driver={...};Dbq=...;`）。
    #[serde(alias = "connectionString")]
    pub connection_string: String,
    pub username: Option<String>,
    pub password: Option<String>,
    #[serde(alias = "encryptedPassword")]
    pub encrypted_password: Option<String>,
}

fn decrypt_password_if_needed(state: &AppState, base: &mut OracleRequestBase) -> Result<(), AppError> {
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
pub struct OracleListColumnsRequest {
    #[serde(flatten)]
    pub base: OracleRequestBase,
    pub table_name: String,
}

#[derive(Debug, Deserialize)]
pub struct OracleQueryRequest {
    #[serde(flatten)]
    pub base: OracleRequestBase,
    pub sql: String,
}

fn build_connection_string(base: &OracleRequestBase) -> Result<String, AppError> {
    let raw = base.connection_string.trim();
    if raw.is_empty() {
        return Err(AppError::Validation("connection_string is required".into()));
    }

    let mut conn_str = raw.to_string();
    if !conn_str.ends_with(';') {
        conn_str.push(';');
    }

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

    for (label, value) in [("username", username), ("password", password)] {
        if let Some(v) = value {
            if v.contains(';') || v.contains('\0') {
                return Err(AppError::Validation(format!("{label} contains invalid characters")));
            }
        }
    }

    if let Some(user) = username {
        conn_str.push_str(&format!("Uid={user};"));
    }
    if let Some(pw) = password {
        conn_str.push_str(&format!("Pwd={pw};"));
    }

    Ok(conn_str)
}

#[cfg(windows)]
const DEFAULT_ROW_LIMIT: usize = 1000;
#[cfg(windows)]
const DEFAULT_TEXT_LIMIT: usize = 4096;
#[cfg(windows)]
const DEFAULT_BATCH_SIZE: usize = 100;

#[cfg(windows)]
fn odbc_error_to_app(error: odbc_api::Error) -> AppError {
    AppError::Database(error.to_string())
}

#[cfg(windows)]
fn odbc_fetch_text_rows(
    mut cursor: impl odbc_api::Cursor,
    max_rows: usize,
) -> Result<(Vec<String>, Vec<Vec<serde_json::Value>>), AppError> {
    use odbc_api::buffers::TextRowSet;

    let columns = cursor
        .column_names()
        .map_err(odbc_error_to_app)?
        .collect::<Result<Vec<_>, _>>()
        .map_err(odbc_error_to_app)?;

    let mut buffers =
        TextRowSet::for_cursor(DEFAULT_BATCH_SIZE, &mut cursor, Some(DEFAULT_TEXT_LIMIT))
            .map_err(odbc_error_to_app)?;
    let mut row_set_cursor = cursor.bind_buffer(&mut buffers).map_err(odbc_error_to_app)?;

    let mut rows: Vec<Vec<serde_json::Value>> = Vec::new();
    while let Some(batch) = row_set_cursor.fetch().map_err(odbc_error_to_app)? {
        for row_index in 0..batch.num_rows() {
            let mut row = Vec::with_capacity(batch.num_cols());
            for col_index in 0..batch.num_cols() {
                let cell = batch.at(col_index, row_index).unwrap_or(&[]);
                let text = String::from_utf8_lossy(cell).to_string();
                row.push(serde_json::Value::String(text));
            }
            rows.push(row);
            if rows.len() >= max_rows {
                return Ok((columns, rows));
            }
        }
    }

    Ok((columns, rows))
}

#[cfg(windows)]
fn oracle_query_blocking(conn_str: String, sql: String) -> Result<DbQueryResult, AppError> {
    use odbc_api::{ConnectionOptions, Environment};

    let env = Environment::new().map_err(odbc_error_to_app)?;
    let conn = env
        .connect_with_connection_string(&conn_str, ConnectionOptions::default())
        .map_err(odbc_error_to_app)?;

    let started = Instant::now();
    let result = conn.execute(&sql, ()).map_err(odbc_error_to_app)?;

    match result {
        Some(cursor) => {
            let (columns, rows) = odbc_fetch_text_rows(cursor, DEFAULT_ROW_LIMIT)?;
            Ok(DbQueryResult {
                columns,
                rows,
                rows_affected: None,
                duration_ms: started.elapsed().as_millis() as u64,
            })
        }
        None => Ok(DbQueryResult {
            columns: vec![],
            rows: vec![],
            rows_affected: None,
            duration_ms: started.elapsed().as_millis() as u64,
        }),
    }
}

#[cfg(windows)]
fn oracle_list_tables_blocking(conn_str: String) -> Result<Vec<DbTable>, AppError> {
    use odbc_api::{ConnectionOptions, Environment};

    let env = Environment::new().map_err(odbc_error_to_app)?;
    let conn = env
        .connect_with_connection_string(&conn_str, ConnectionOptions::default())
        .map_err(odbc_error_to_app)?;

    let sql = "SELECT table_name AS name, 'table' AS kind FROM user_tables \
               UNION ALL \
               SELECT view_name AS name, 'view' AS kind FROM user_views \
               ORDER BY kind, name";

    let mut items = Vec::new();
    if let Some(cursor) = conn.execute(sql, ()).map_err(odbc_error_to_app)? {
        let (_columns, rows) = odbc_fetch_text_rows(cursor, DEFAULT_ROW_LIMIT)?;
        for row in rows {
            let name = row
                .get(0)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let kind = row
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("table")
                .trim()
                .to_string();
            if !name.is_empty() {
                items.push(DbTable { name, kind });
            }
        }
    }

    Ok(items)
}

#[cfg(windows)]
fn oracle_list_columns_blocking(conn_str: String, table_name: String) -> Result<Vec<DbColumn>, AppError> {
    use odbc_api::{ConnectionOptions, Environment, IntoParameter};

    let env = Environment::new().map_err(odbc_error_to_app)?;
    let conn = env
        .connect_with_connection_string(&conn_str, ConnectionOptions::default())
        .map_err(odbc_error_to_app)?;

    let table_upper = table_name.trim().to_uppercase();
    if table_upper.is_empty() {
        return Err(AppError::Validation("table_name is required".into()));
    }

    let sql = "SELECT c.column_id AS cid, c.column_name AS name, c.data_type AS data_type, \
               CASE WHEN c.nullable = 'N' THEN 1 ELSE 0 END AS not_null, \
               c.data_default AS default_value, \
               CASE WHEN pk.column_name IS NOT NULL THEN 1 ELSE 0 END AS primary_key \
               FROM user_tab_columns c \
               LEFT JOIN ( \
                 SELECT acc.column_name \
                 FROM user_constraints ac \
                 JOIN user_cons_columns acc ON ac.constraint_name = acc.constraint_name \
                 WHERE ac.constraint_type = 'P' AND ac.table_name = ? \
               ) pk ON pk.column_name = c.column_name \
               WHERE c.table_name = ? \
               ORDER BY c.column_id";

    let p1 = table_upper.as_str().into_parameter();
    let p2 = table_upper.as_str().into_parameter();

    let mut items = Vec::new();
    if let Some(cursor) = conn
        .execute(sql, (&p1, &p2))
        .map_err(odbc_error_to_app)?
    {
        let (_columns, rows) = odbc_fetch_text_rows(cursor, DEFAULT_ROW_LIMIT)?;
        for row in rows {
            let cid = row
                .get(0)
                .and_then(|v| v.as_str())
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or_default();
            let name = row
                .get(1)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let data_type = row
                .get(2)
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .trim()
                .to_string();
            let not_null = row
                .get(3)
                .and_then(|v| v.as_str())
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(0)
                != 0;
            let default_value = row
                .get(4)
                .and_then(|v| v.as_str())
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string);
            let primary_key = row
                .get(5)
                .and_then(|v| v.as_str())
                .and_then(|s| s.trim().parse::<i32>().ok())
                .unwrap_or(0)
                != 0;

            if !name.is_empty() {
                items.push(DbColumn {
                    cid,
                    name,
                    data_type,
                    not_null,
                    default_value,
                    primary_key,
                });
            }
        }
    }

    Ok(items)
}

#[tauri::command]
pub async fn db_oracle_list_tables(
    state: State<'_, AppState>,
    mut req: OracleRequestBase,
) -> CmdResult<Vec<DbTable>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req)?;
    let conn_str = build_connection_string(&req)?;

    #[cfg(windows)]
    {
        return tokio::task::spawn_blocking(move || oracle_list_tables_blocking(conn_str))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    #[cfg(not(windows))]
    {
        let _ = conn_str;
        Err(AppError::Validation(
            "Oracle ODBC is only supported on Windows in this build".into(),
        ))
    }
}

#[tauri::command]
pub async fn db_oracle_list_columns(
    state: State<'_, AppState>,
    mut req: OracleListColumnsRequest,
) -> CmdResult<Vec<DbColumn>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let conn_str = build_connection_string(&req.base)?;
    let table = req.table_name;

    #[cfg(windows)]
    {
        return tokio::task::spawn_blocking(move || oracle_list_columns_blocking(conn_str, table))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    #[cfg(not(windows))]
    {
        let _ = (conn_str, table);
        Err(AppError::Validation(
            "Oracle ODBC is only supported on Windows in this build".into(),
        ))
    }
}

#[tauri::command]
pub async fn db_oracle_query(
    state: State<'_, AppState>,
    mut req: OracleQueryRequest,
) -> CmdResult<DbQueryResult> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let sql = req.sql.trim();
    if sql.is_empty() {
        return Err(AppError::Validation("sql is required".into()));
    }

    let conn_str = build_connection_string(&req.base)?;
    let sql = sql.to_string();

    #[cfg(windows)]
    {
        return tokio::task::spawn_blocking(move || oracle_query_blocking(conn_str, sql))
            .await
            .map_err(|e| AppError::Database(e.to_string()))?;
    }

    #[cfg(not(windows))]
    {
        let _ = (conn_str, sql);
        Err(AppError::Validation(
            "Oracle ODBC is only supported on Windows in this build".into(),
        ))
    }
}

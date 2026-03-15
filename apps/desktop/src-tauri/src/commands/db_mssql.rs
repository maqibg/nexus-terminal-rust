use api_contract::error::{AppError, CmdResult};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Deserialize;
use std::time::Instant;
use tauri::State;
use tokio::net::TcpStream;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};
use tiberius::{AuthMethod, Client, ColumnData, Config, EncryptionLevel};

use crate::state::AppState;

use super::db_types::{DbColumn, DbQueryResult, DbTable};

type MssqlClient = Client<Compat<TcpStream>>;

#[derive(Debug, Deserialize)]
pub struct MssqlRequestBase {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub encrypted_password: Option<String>,
    pub database: String,
    pub schema: Option<String>,
    /// 是否信任服务器证书（自签名等）。默认 false。
    #[serde(alias = "trustServerCertificate")]
    pub trust_server_certificate: Option<bool>,
    /// 是否开启加密。默认使用驱动默认值（当前为 Required）。
    pub encrypt: Option<bool>,
}

fn decrypt_password_if_needed(state: &AppState, base: &mut MssqlRequestBase) -> Result<(), AppError> {
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
pub struct MssqlListColumnsRequest {
    #[serde(flatten)]
    pub base: MssqlRequestBase,
    pub table_name: String,
}

#[derive(Debug, Deserialize)]
pub struct MssqlQueryRequest {
    #[serde(flatten)]
    pub base: MssqlRequestBase,
    pub sql: String,
}

async fn connect_mssql(base: &MssqlRequestBase) -> Result<MssqlClient, AppError> {
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

    let mut config = Config::new();
    config.host(host);
    config.port(base.port);
    config.database(database);
    config.application_name("nexus-terminal");

    let password = base.password.as_deref().unwrap_or_default();
    config.authentication(AuthMethod::sql_server(username, password));

    if base.trust_server_certificate.unwrap_or(false) {
        config.trust_cert();
    }
    if let Some(encrypt) = base.encrypt {
        config.encryption(if encrypt {
            EncryptionLevel::Required
        } else {
            EncryptionLevel::Off
        });
    }

    let tcp = TcpStream::connect(config.get_addr())
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;
    tcp.set_nodelay(true)
        .map_err(|error| AppError::Database(error.to_string()))?;

    Client::connect(config, tcp.compat_write())
        .await
        .map_err(|error| AppError::Database(error.to_string()))
}

fn mssql_schema(base: &MssqlRequestBase) -> String {
    base.schema
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .unwrap_or("dbo")
        .to_string()
}

fn mssql_table_kind(raw: &str) -> String {
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
        "select" | "with" | "show" | "desc" | "describe" | "explain" | "exec" | "execute"
    )
}

fn mssql_cell_to_json(cell: &ColumnData<'static>) -> serde_json::Value {
    use serde_json::{Number, Value};

    match cell {
        ColumnData::U8(v) => v.map(Value::from).unwrap_or(Value::Null),
        ColumnData::I16(v) => v.map(Value::from).unwrap_or(Value::Null),
        ColumnData::I32(v) => v.map(Value::from).unwrap_or(Value::Null),
        ColumnData::I64(v) => v.map(Value::from).unwrap_or(Value::Null),
        ColumnData::F32(v) => v
            .and_then(|n| Number::from_f64(n as f64))
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ColumnData::F64(v) => v
            .and_then(Number::from_f64)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ColumnData::Bit(v) => v.map(Value::from).unwrap_or(Value::Null),
        ColumnData::String(v) => v
            .as_ref()
            .map(|s| Value::String(s.to_string()))
            .unwrap_or(Value::Null),
        ColumnData::Guid(v) => v
            .as_ref()
            .map(|id| Value::String(id.to_string()))
            .unwrap_or(Value::Null),
        ColumnData::Binary(v) => v
            .as_ref()
            .map(|b| Value::String(format!("base64:{}", STANDARD.encode(b.as_ref()))))
            .unwrap_or(Value::Null),
        ColumnData::Numeric(v) => v
            .as_ref()
            .map(|n| Value::String(n.to_string()))
            .unwrap_or(Value::Null),
        ColumnData::Xml(v) => v
            .as_ref()
            .map(|x| Value::String(x.to_string()))
            .unwrap_or(Value::Null),
        ColumnData::DateTime(v) => v
            .as_ref()
            .map(|dt| Value::String(format!("{dt:?}")))
            .unwrap_or(Value::Null),
        ColumnData::SmallDateTime(v) => v
            .as_ref()
            .map(|dt| Value::String(format!("{dt:?}")))
            .unwrap_or(Value::Null),
        ColumnData::Time(v) => v
            .as_ref()
            .map(|dt| Value::String(format!("{dt:?}")))
            .unwrap_or(Value::Null),
        ColumnData::Date(v) => v
            .as_ref()
            .map(|dt| Value::String(format!("{dt:?}")))
            .unwrap_or(Value::Null),
        ColumnData::DateTime2(v) => v
            .as_ref()
            .map(|dt| Value::String(format!("{dt:?}")))
            .unwrap_or(Value::Null),
        ColumnData::DateTimeOffset(v) => v
            .as_ref()
            .map(|dt| Value::String(format!("{dt:?}")))
            .unwrap_or(Value::Null),
    }
}

fn mssql_row_to_json(row: &tiberius::Row) -> Vec<serde_json::Value> {
    row.cells()
        .map(|(_, cell)| mssql_cell_to_json(cell))
        .collect()
}

async fn mssql_run_read_query(
    client: &mut MssqlClient,
    sql: &str,
    started: Instant,
) -> Result<DbQueryResult, AppError> {
    let mut stream = client
        .query(sql, &[])
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    let columns = stream
        .columns()
        .await
        .map_err(|error| AppError::Database(error.to_string()))?
        .map(|cols| cols.iter().map(|c| c.name().to_string()).collect())
        .unwrap_or_else(Vec::new);

    let rows = stream
        .into_first_result()
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;
    let out_rows = rows.iter().map(mssql_row_to_json).collect();

    Ok(DbQueryResult {
        columns,
        rows: out_rows,
        rows_affected: None,
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

async fn mssql_run_exec(
    client: &mut MssqlClient,
    sql: &str,
    started: Instant,
) -> Result<DbQueryResult, AppError> {
    let result = client
        .execute(sql, &[])
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    Ok(DbQueryResult {
        columns: vec![],
        rows: vec![],
        rows_affected: Some(result.rows_affected().iter().copied().sum::<u64>()),
        duration_ms: started.elapsed().as_millis() as u64,
    })
}

#[tauri::command]
pub async fn db_mssql_list_tables(
    state: State<'_, AppState>,
    mut req: MssqlRequestBase,
) -> CmdResult<Vec<DbTable>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req)?;
    let schema = mssql_schema(&req);
    let mut client = connect_mssql(&req).await?;

    let stream = client
        .query(
        "SELECT TABLE_NAME AS name, TABLE_TYPE AS kind \
         FROM INFORMATION_SCHEMA.TABLES \
         WHERE TABLE_SCHEMA = @P1 AND TABLE_TYPE IN ('BASE TABLE', 'VIEW') \
         ORDER BY TABLE_TYPE, TABLE_NAME",
            &[&schema],
        )
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let name = row.get::<&str, _>("name").unwrap_or("").to_string();
        let raw_kind = row.get::<&str, _>("kind").unwrap_or("BASE TABLE");
        items.push(DbTable {
            name,
            kind: mssql_table_kind(raw_kind),
        });
    }
    Ok(items)
}

#[tauri::command]
pub async fn db_mssql_list_columns(
    state: State<'_, AppState>,
    mut req: MssqlListColumnsRequest,
) -> CmdResult<Vec<DbColumn>> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let schema = mssql_schema(&req.base);
    let table = req.table_name.trim();
    if table.is_empty() {
        return Err(AppError::Validation("table_name is required".into()));
    }

    let table = table.to_string();
    let mut client = connect_mssql(&req.base).await?;
    let stream = client
        .query(
        "SELECT c.ORDINAL_POSITION AS cid, c.COLUMN_NAME AS name, c.DATA_TYPE AS data_type, \
         c.IS_NULLABLE AS is_nullable, c.COLUMN_DEFAULT AS default_value, \
         CASE WHEN tc.CONSTRAINT_TYPE = 'PRIMARY KEY' THEN 1 ELSE 0 END AS primary_key \
         FROM INFORMATION_SCHEMA.COLUMNS c \
         LEFT JOIN INFORMATION_SCHEMA.KEY_COLUMN_USAGE kcu \
           ON kcu.TABLE_SCHEMA=c.TABLE_SCHEMA AND kcu.TABLE_NAME=c.TABLE_NAME AND kcu.COLUMN_NAME=c.COLUMN_NAME \
         LEFT JOIN INFORMATION_SCHEMA.TABLE_CONSTRAINTS tc \
           ON tc.CONSTRAINT_NAME=kcu.CONSTRAINT_NAME AND tc.TABLE_SCHEMA=kcu.TABLE_SCHEMA AND tc.TABLE_NAME=kcu.TABLE_NAME \
         WHERE c.TABLE_SCHEMA=@P1 AND c.TABLE_NAME=@P2 \
         ORDER BY c.ORDINAL_POSITION",
            &[&schema, &table],
        )
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    let rows = stream
        .into_first_result()
        .await
        .map_err(|error| AppError::Database(error.to_string()))?;

    let mut items = Vec::with_capacity(rows.len());
    for row in rows {
        let cid = row.get::<i32, _>("cid").unwrap_or_default();
        let name = row.get::<&str, _>("name").unwrap_or("").to_string();
        let data_type = row
            .get::<&str, _>("data_type")
            .unwrap_or("")
            .to_string();
        let nullable = row.get::<&str, _>("is_nullable").unwrap_or("YES");
        let default_value = row.get::<&str, _>("default_value").map(|v| v.to_string());
        let primary_key_raw = row.get::<i32, _>("primary_key").unwrap_or_default();

        items.push(DbColumn {
            cid,
            name,
            data_type,
            not_null: nullable.eq_ignore_ascii_case("NO"),
            default_value,
            primary_key: primary_key_raw != 0,
        });
    }
    Ok(items)
}

#[tauri::command]
pub async fn db_mssql_query(
    state: State<'_, AppState>,
    mut req: MssqlQueryRequest,
) -> CmdResult<DbQueryResult> {
    state.auth.require_auth().await?;

    decrypt_password_if_needed(&state, &mut req.base)?;
    let sql = req.sql.trim();
    if sql.is_empty() {
        return Err(AppError::Validation("sql is required".into()));
    }

    let mut client = connect_mssql(&req.base).await?;
    let started = Instant::now();
    if is_read_query(sql) {
        return mssql_run_read_query(&mut client, sql, started).await;
    }
    mssql_run_exec(&mut client, sql, started).await
}

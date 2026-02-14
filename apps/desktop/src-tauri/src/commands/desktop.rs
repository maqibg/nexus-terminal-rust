//! Desktop-only system integration commands (RDP, shell launch).

use api_contract::error::AppError;
use connection_core::repository::ConnectionRepository;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

type CmdResult<T> = Result<T, AppError>;

#[derive(Deserialize)]
pub struct DesktopOpenRdpRequest {
    pub host: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[tauri::command]
pub async fn desktop_open_rdp(
    state: State<'_, AppState>,
    req: DesktopOpenRdpRequest,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    open_rdp_on_platform(
        req.host.trim(),
        req.port,
        req.username.as_deref(),
        req.password.as_deref(),
    )
}

#[tauri::command]
pub async fn desktop_open_rdp_connection(
    state: State<'_, AppState>,
    connection_id: i64,
) -> CmdResult<()> {
    state.auth.require_auth().await?;
    let conn = state
        .conn_repo
        .get_connection(connection_id)
        .await
        .map_err(AppError::Database)?
        .ok_or(AppError::NotFound("connection not found".into()))?;

    if !conn.conn_type.eq_ignore_ascii_case("RDP") {
        return Err(AppError::Validation(
            "only RDP connections can be opened with mstsc".into(),
        ));
    }

    let password = conn
        .encrypted_password
        .as_deref()
        .map(|encrypted| state.crypto.decrypt(encrypted))
        .transpose()
        .map_err(|e| AppError::Crypto(e.to_string()))?;

    let port = u16::try_from(conn.port).ok().filter(|v| *v > 0);

    open_rdp_on_platform(
        conn.host.trim(),
        port,
        Some(conn.username.as_str()),
        password.as_deref(),
    )
}

#[cfg(not(windows))]
fn open_rdp_on_platform(
    _host: &str,
    _port: Option<u16>,
    _username: Option<&str>,
    _password: Option<&str>,
) -> CmdResult<()> {
    Err(AppError::Validation(
        "RDP launch is only supported on Windows desktop".into(),
    ))
}

#[cfg(windows)]
fn open_rdp_on_platform(
    host: &str,
    port: Option<u16>,
    username: Option<&str>,
    password: Option<&str>,
) -> CmdResult<()> {
    use std::os::windows::process::CommandExt;
    use std::process::Command;

    const DETACHED_PROCESS: u32 = 0x00000008;

    if host.is_empty() {
        return Err(AppError::Validation("host is required".into()));
    }

    let server = match port {
        Some(v) if v > 0 => format!("{host}:{v}"),
        _ => host.to_string(),
    };

    let mut credentials_added = false;
    if let (Some(user), Some(pass)) = (username, password) {
        let status = Command::new("cmdkey.exe")
            .arg(format!("/generic:TERMSRV/{host}"))
            .arg(format!("/user:{user}"))
            .arg(format!("/pass:{pass}"))
            .status()
            .map_err(|e| AppError::Internal(format!("failed to execute cmdkey: {e}")))?;

        if !status.success() {
            return Err(AppError::Internal(
                "cmdkey rejected stored credentials".into(),
            ));
        }
        credentials_added = true;
    }

    let launch_result = Command::new("mstsc.exe")
        .arg(format!("/v:{server}"))
        .creation_flags(DETACHED_PROCESS)
        .spawn()
        .map(|_| ())
        .map_err(|e| AppError::Internal(format!("failed to launch mstsc: {e}")));

    if launch_result.is_err() && credentials_added {
        let _ = delete_cmdkey(host);
    }

    if credentials_added {
        let target = host.to_string();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let _ = delete_cmdkey(&target);
        });
    }

    launch_result
}

#[cfg(windows)]
fn delete_cmdkey(host: &str) -> Result<(), String> {
    let status = std::process::Command::new("cmdkey.exe")
        .arg(format!("/delete:TERMSRV/{host}"))
        .status()
        .map_err(|e| e.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("cmdkey delete failed".into())
    }
}

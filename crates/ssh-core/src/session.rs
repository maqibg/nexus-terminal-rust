//! SSH session management using russh.
//!
//! Each SshSession wraps a russh Channel, providing:
//! - connect (password / key auth)
//! - open shell with PTY
//! - write stdin, resize, close
//! - async stdout read via channel.wait()

use russh::keys::*;
use russh::*;
use russh_keys::key::PrivateKeyWithHashAlg;
use std::sync::Arc;
use tracing::info;

/// Minimal russh client handler.
pub struct SshHandler;

#[async_trait::async_trait]
impl client::Handler for SshHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _server_public_key: &ssh_key::PublicKey,
    ) -> Result<bool, Self::Error> {
        // Desktop app: accept all host keys (user-trusted connections)
        Ok(true)
    }
}

/// Decoded connection credentials for SSH.
pub struct SshCredentials {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
}

pub enum SshAuth {
    Password(String),
    Key {
        private_key_pem: String,
        passphrase: Option<String>,
    },
}

/// Connect to SSH server and authenticate. Returns the Handle for opening channels.
pub async fn connect_and_authenticate(
    creds: &SshCredentials,
) -> Result<client::Handle<SshHandler>, String> {
    let config = Arc::new(client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(30)),
        keepalive_interval: Some(std::time::Duration::from_secs(5)),
        keepalive_max: 10,
        ..Default::default()
    });

    let mut handle = client::connect(config, (creds.host.as_str(), creds.port), SshHandler)
        .await
        .map_err(|e| format!("SSH connect failed: {e}"))?;

    let auth_ok = match &creds.auth {
        SshAuth::Password(pw) => handle
            .authenticate_password(&creds.username, pw)
            .await
            .map_err(|e| format!("auth failed: {e}"))?,
        SshAuth::Key {
            private_key_pem,
            passphrase,
        } => {
            let key_pair = decode_secret_key(private_key_pem, passphrase.as_deref())
                .map_err(|e| format!("key decode failed: {e}"))?;
            let key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key_pair), None)
                .map_err(|e| format!("key hash failed: {e}"))?;
            handle
                .authenticate_publickey(&creds.username, key_with_hash)
                .await
                .map_err(|e| format!("pubkey auth failed: {e}"))?
        }
    };

    if !auth_ok {
        return Err("authentication rejected".into());
    }

    info!(host = %creds.host, port = creds.port, "SSH authenticated");
    Ok(handle)
}

/// Connect to SSH server, authenticate, open PTY + shell.
/// Returns the Channel for reading/writing.
pub async fn connect_and_open_shell(
    creds: &SshCredentials,
    cols: u32,
    rows: u32,
    term: &str,
) -> Result<Channel<client::Msg>, String> {
    let handle = connect_and_authenticate(creds).await?;

    open_shell_channel(&handle, cols, rows, term).await
}

/// Open a PTY + shell channel on an existing authenticated handle.
pub async fn open_shell_channel(
    handle: &client::Handle<SshHandler>,
    cols: u32,
    rows: u32,
    term: &str,
) -> Result<Channel<client::Msg>, String> {
    let channel = handle
        .channel_open_session()
        .await
        .map_err(|e| format!("channel open failed: {e}"))?;

    channel
        .request_pty(false, term, cols, rows, 0, 0, &[])
        .await
        .map_err(|e| format!("pty request failed: {e}"))?;

    channel
        .request_shell(false)
        .await
        .map_err(|e| format!("shell request failed: {e}"))?;

    info!("Shell opened ({}x{})", cols, rows);
    Ok(channel)
}
/// Connect to SSH server, authenticate, open SFTP subsystem channel.
pub async fn connect_and_open_sftp(creds: &SshCredentials) -> Result<Channel<client::Msg>, String> {
    let handle = connect_and_authenticate(creds).await?;

    let channel = handle
        .channel_open_session()
        .await
        .map_err(|e| format!("channel open failed: {e}"))?;

    channel
        .request_subsystem(false, "sftp")
        .await
        .map_err(|e| format!("sftp subsystem request failed: {e}"))?;

    info!("SFTP channel opened");
    Ok(channel)
}

/// Jump host credentials for multi-hop SSH.
pub struct JumpHost {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SshAuth,
}

/// Connect through a chain of jump hosts, then authenticate to the final target.
/// Returns a Handle to the final destination.
pub async fn connect_via_jump_chain(
    jumps: &[JumpHost],
    target: &SshCredentials,
) -> Result<client::Handle<SshHandler>, String> {
    if jumps.is_empty() {
        return connect_and_authenticate(target).await;
    }

    // Connect to first jump host
    let first = &jumps[0];
    let first_creds = SshCredentials {
        host: first.host.clone(),
        port: first.port,
        username: first.username.clone(),
        auth: match &first.auth {
            SshAuth::Password(p) => SshAuth::Password(p.clone()),
            SshAuth::Key {
                private_key_pem,
                passphrase,
            } => SshAuth::Key {
                private_key_pem: private_key_pem.clone(),
                passphrase: passphrase.clone(),
            },
        },
    };
    let mut current_handle = connect_and_authenticate(&first_creds).await?;

    // Chain through remaining jump hosts
    for jump in &jumps[1..] {
        let forwarded = current_handle
            .channel_open_direct_tcpip(&jump.host, jump.port as u32, "127.0.0.1", 0)
            .await
            .map_err(|e| format!("jump forward failed: {e}"))?;

        let config = Arc::new(client::Config {
            inactivity_timeout: Some(std::time::Duration::from_secs(30)),
            keepalive_interval: Some(std::time::Duration::from_secs(5)),
            keepalive_max: 10,
            ..Default::default()
        });

        let mut handle = client::connect_stream(config, forwarded.into_stream(), SshHandler)
            .await
            .map_err(|e| format!("jump connect failed: {e}"))?;

        let auth_ok = authenticate_handle(&mut handle, &jump.username, &jump.auth).await?;
        if !auth_ok {
            return Err("jump auth rejected".into());
        }

        current_handle = handle;
        info!(host = %jump.host, port = jump.port, "Jump hop authenticated");
    }

    // Forward to final target through last jump
    let forwarded = current_handle
        .channel_open_direct_tcpip(&target.host, target.port as u32, "127.0.0.1", 0)
        .await
        .map_err(|e| format!("final forward failed: {e}"))?;

    let config = Arc::new(client::Config {
        inactivity_timeout: Some(std::time::Duration::from_secs(30)),
        keepalive_interval: Some(std::time::Duration::from_secs(5)),
        keepalive_max: 10,
        ..Default::default()
    });

    let mut final_handle = client::connect_stream(config, forwarded.into_stream(), SshHandler)
        .await
        .map_err(|e| format!("target connect failed: {e}"))?;

    let auth_ok = authenticate_handle(&mut final_handle, &target.username, &target.auth).await?;
    if !auth_ok {
        return Err("target auth rejected".into());
    }

    info!(host = %target.host, port = target.port, "Jump chain complete");
    Ok(final_handle)
}

/// Authenticate a handle with given credentials.
async fn authenticate_handle(
    handle: &mut client::Handle<SshHandler>,
    username: &str,
    auth: &SshAuth,
) -> Result<bool, String> {
    match auth {
        SshAuth::Password(pw) => handle
            .authenticate_password(username, pw)
            .await
            .map_err(|e| format!("auth failed: {e}")),
        SshAuth::Key {
            private_key_pem,
            passphrase,
        } => {
            let key_pair = decode_secret_key(private_key_pem, passphrase.as_deref())
                .map_err(|e| format!("key decode failed: {e}"))?;
            let key_with_hash = PrivateKeyWithHashAlg::new(Arc::new(key_pair), None)
                .map_err(|e| format!("key hash failed: {e}"))?;
            handle
                .authenticate_publickey(username, key_with_hash)
                .await
                .map_err(|e| format!("pubkey auth failed: {e}"))
        }
    }
}

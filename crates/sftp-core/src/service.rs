//! SFTP operations wrapping russh-sftp.

use russh_sftp::client::SftpSession;
use serde::Serialize;

/// SFTP file entry returned to frontend.
#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    pub modified: Option<u64>,
    pub permissions: Option<u32>,
}

/// List directory contents.
pub async fn list_dir(sftp: &SftpSession, path: &str) -> Result<Vec<FileEntry>, String> {
    let entries = sftp
        .read_dir(path)
        .await
        .map_err(|e| format!("readdir failed: {e}"))?;
    let mut result = Vec::new();
    for entry in entries {
        let name = entry.file_name();
        if name == "." || name == ".." {
            continue;
        }
        let full_path = if path.ends_with('/') {
            format!("{path}{name}")
        } else {
            format!("{path}/{name}")
        };
        let attrs = &entry.metadata();
        result.push(FileEntry {
            name,
            path: full_path,
            is_dir: attrs.is_dir(),
            size: attrs.size.unwrap_or(0),
            modified: attrs.mtime.map(|v| v as u64),
            permissions: attrs.permissions,
        });
    }
    Ok(result)
}

/// Read file contents as bytes.
pub async fn read_file(sftp: &SftpSession, path: &str) -> Result<Vec<u8>, String> {
    let mut file = sftp
        .open(path)
        .await
        .map_err(|e| format!("open failed: {e}"))?;
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .await
        .map_err(|e| format!("read failed: {e}"))?;
    Ok(buf)
}

/// Write bytes to a file (create/overwrite).
pub async fn write_file(sftp: &SftpSession, path: &str, data: &[u8]) -> Result<(), String> {
    let mut file = sftp
        .create(path)
        .await
        .map_err(|e| format!("create failed: {e}"))?;
    use tokio::io::AsyncWriteExt;
    file.write_all(data)
        .await
        .map_err(|e| format!("write failed: {e}"))?;
    file.shutdown()
        .await
        .map_err(|e| format!("flush failed: {e}"))?;
    Ok(())
}

/// Delete a file.
pub async fn remove_file(sftp: &SftpSession, path: &str) -> Result<(), String> {
    sftp.remove_file(path)
        .await
        .map_err(|e| format!("remove failed: {e}"))
}

/// Create a directory.
pub async fn mkdir(sftp: &SftpSession, path: &str) -> Result<(), String> {
    sftp.create_dir(path)
        .await
        .map_err(|e| format!("mkdir failed: {e}"))
}

/// Remove a directory.
pub async fn rmdir(sftp: &SftpSession, path: &str) -> Result<(), String> {
    sftp.remove_dir(path)
        .await
        .map_err(|e| format!("rmdir failed: {e}"))
}

/// Rename/move a file or directory.
pub async fn rename(sftp: &SftpSession, old_path: &str, new_path: &str) -> Result<(), String> {
    sftp.rename(old_path, new_path)
        .await
        .map_err(|e| format!("rename failed: {e}"))
}

/// Get file/dir metadata.
pub async fn stat(sftp: &SftpSession, path: &str) -> Result<FileEntry, String> {
    let attrs = sftp
        .metadata(path)
        .await
        .map_err(|e| format!("stat failed: {e}"))?;
    let name = path.rsplit('/').next().unwrap_or(path).to_string();
    Ok(FileEntry {
        name,
        path: path.to_string(),
        is_dir: attrs.is_dir(),
        size: attrs.size.unwrap_or(0),
        modified: attrs.mtime.map(|v| v as u64),
        permissions: attrs.permissions,
    })
}

/// Change file permissions.
pub async fn chmod(sftp: &SftpSession, path: &str, mode: u32) -> Result<(), String> {
    use russh_sftp::protocol::FileAttributes;
    let attrs = FileAttributes {
        permissions: Some(mode),
        ..Default::default()
    };
    sftp.set_metadata(path, attrs)
        .await
        .map_err(|e| format!("chmod failed: {e}"))
}

/// Append bytes to an existing file.
pub async fn append_file(sftp: &SftpSession, path: &str, data: &[u8]) -> Result<(), String> {
    use russh_sftp::protocol::OpenFlags;
    use tokio::io::AsyncWriteExt;
    let mut file = sftp
        .open_with_flags(path, OpenFlags::WRITE | OpenFlags::APPEND)
        .await
        .map_err(|e| format!("open for append failed: {e}"))?;
    file.write_all(data)
        .await
        .map_err(|e| format!("append failed: {e}"))?;
    file.shutdown()
        .await
        .map_err(|e| format!("flush failed: {e}"))?;
    Ok(())
}

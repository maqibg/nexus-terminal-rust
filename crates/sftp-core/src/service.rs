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

/// Recursively remove a directory and all of its children.
pub async fn rmdir_recursive(sftp: &SftpSession, path: &str) -> Result<(), String> {
    let mut directory_stack = vec![path.to_string()];
    let mut removal_order = Vec::new();

    while let Some(current_dir) = directory_stack.pop() {
        removal_order.push(current_dir.clone());
        let entries = list_dir(sftp, &current_dir).await?;
        for entry in entries {
            if entry.is_dir {
                directory_stack.push(entry.path);
            } else {
                remove_file(sftp, &entry.path).await?;
            }
        }
    }

    for directory in removal_order.into_iter().rev() {
        rmdir(sftp, &directory).await?;
    }

    Ok(())
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

/// Stream-copy a file within the same SFTP session to avoid loading the whole
/// file into frontend memory.
pub async fn copy_file_streaming(
    sftp: &SftpSession,
    source_path: &str,
    target_path: &str,
) -> Result<(), String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut source = sftp
        .open(source_path)
        .await
        .map_err(|e| format!("open source failed: {e}"))?;
    let mut target = sftp
        .create(target_path)
        .await
        .map_err(|e| format!("create target failed: {e}"))?;

    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .await
            .map_err(|e| format!("read source failed: {e}"))?;
        if read == 0 {
            break;
        }

        target
            .write_all(&buffer[..read])
            .await
            .map_err(|e| format!("write target failed: {e}"))?;
    }

    target
        .shutdown()
        .await
        .map_err(|e| format!("flush target failed: {e}"))?;
    Ok(())
}

//! SFTP operations wrapping russh-sftp.
//!
//! Every failure is described through [`describe`], which prefixes a stable
//! category token and the affected path. The frontend branches on that token to
//! tell "permission denied" apart from "connection lost" without parsing the
//! server's natural-language message.

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

/// Coarse category for an SFTP failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SftpErrorKind {
    NotFound,
    PermissionDenied,
    Connection,
    Protocol,
    TooLarge,
    Other,
}

impl SftpErrorKind {
    pub fn as_token(self) -> &'static str {
        match self {
            Self::NotFound => "not-found",
            Self::PermissionDenied => "permission-denied",
            Self::Connection => "connection",
            Self::Protocol => "protocol",
            Self::TooLarge => "too-large",
            Self::Other => "other",
        }
    }
}

/// Map a raw server message onto a category.
pub fn classify(error: &str) -> SftpErrorKind {
    let lower = error.to_lowercase();

    if lower.contains("no such file")
        || lower.contains("not found")
        || lower.contains("does not exist")
        || lower.contains("nosuchfile")
    {
        SftpErrorKind::NotFound
    } else if lower.contains("permission denied") || lower.contains("access denied") {
        SftpErrorKind::PermissionDenied
    } else if lower.contains("connection")
        || lower.contains("broken pipe")
        || lower.contains("not connected")
        || lower.contains("channel closed")
        || lower.contains("unexpected eof")
    {
        SftpErrorKind::Connection
    } else if lower.contains("unsupported")
        || lower.contains("bad message")
        || lower.contains("failure")
        || lower.contains("protocol")
    {
        SftpErrorKind::Protocol
    } else {
        SftpErrorKind::Other
    }
}

/// `<token>: <operation> failed at '<path>': <detail>`
pub fn describe_kind(
    kind: SftpErrorKind,
    operation: &str,
    path: &str,
    detail: &str,
) -> String {
    format!(
        "{}: {} failed at '{}': {}",
        kind.as_token(),
        operation,
        path,
        detail
    )
}

/// Describe an error while deriving its category from the message itself.
pub fn describe(operation: &str, path: &str, error: &str) -> String {
    describe_kind(classify(error), operation, path, error)
}

/// List directory contents.
pub async fn list_dir(sftp: &SftpSession, path: &str) -> Result<Vec<FileEntry>, String> {
    let entries = sftp
        .read_dir(path)
        .await
        .map_err(|e| describe("readdir", path, &e.to_string()))?;
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
        .map_err(|e| describe("open", path, &e.to_string()))?;
    use tokio::io::AsyncReadExt;
    let mut buf = Vec::new();
    file.read_to_end(&mut buf)
        .await
        .map_err(|e| describe("read", path, &e.to_string()))?;
    Ok(buf)
}

/// Read a file for the editor, refusing anything larger than `max_bytes`.
/// Without the guard a stray multi-gigabyte file would be materialised in
/// memory several times over (bytes, base64, IPC payload).
pub async fn read_file_limited(
    sftp: &SftpSession,
    path: &str,
    max_bytes: u64,
) -> Result<Vec<u8>, String> {
    let attrs = sftp
        .metadata(path)
        .await
        .map_err(|e| describe("stat", path, &e.to_string()))?;
    if let Some(size) = attrs.size {
        if size > max_bytes {
            return Err(describe_kind(
                SftpErrorKind::TooLarge,
                "read",
                path,
                &format!("file is {size} bytes, above the {max_bytes} byte editor limit"),
            ));
        }
    }

    use tokio::io::AsyncReadExt;
    let file = sftp
        .open(path)
        .await
        .map_err(|e| describe("open", path, &e.to_string()))?;
    let mut limited = file.take(max_bytes);
    let mut buf = Vec::new();
    limited
        .read_to_end(&mut buf)
        .await
        .map_err(|e| describe("read", path, &e.to_string()))?;
    Ok(buf)
}

/// Write bytes to a file (create/overwrite).
pub async fn write_file(sftp: &SftpSession, path: &str, data: &[u8]) -> Result<(), String> {
    let mut file = sftp
        .create(path)
        .await
        .map_err(|e| describe("create", path, &e.to_string()))?;
    use tokio::io::AsyncWriteExt;
    file.write_all(data)
        .await
        .map_err(|e| describe("write", path, &e.to_string()))?;
    file.shutdown()
        .await
        .map_err(|e| describe("flush", path, &e.to_string()))?;
    Ok(())
}

/// Delete a file.
pub async fn remove_file(sftp: &SftpSession, path: &str) -> Result<(), String> {
    sftp.remove_file(path)
        .await
        .map_err(|e| describe("remove", path, &e.to_string()))
}

/// Create a directory.
pub async fn mkdir(sftp: &SftpSession, path: &str) -> Result<(), String> {
    sftp.create_dir(path)
        .await
        .map_err(|e| describe("mkdir", path, &e.to_string()))
}

/// Remove a directory.
pub async fn rmdir(sftp: &SftpSession, path: &str) -> Result<(), String> {
    sftp.remove_dir(path)
        .await
        .map_err(|e| describe("rmdir", path, &e.to_string()))
}

const MAX_RECURSIVE_DEPTH: usize = 64;

/// Reject paths that make a recursive walk unsafe: the filesystem root, empty
/// or relative paths, and any `..` segment. Interactive callers must never be
/// able to hand us something that walks upwards or spans the whole tree.
fn validate_recursive_target(path: &str) -> Result<(), String> {
    let trimmed = path.trim();
    if trimmed.is_empty() || !trimmed.starts_with('/') {
        return Err(describe_kind(
            SftpErrorKind::Other,
            "rmdir",
            path,
            "refusing non-absolute path",
        ));
    }

    let normalized = trimmed.trim_end_matches('/');
    if normalized.is_empty() {
        return Err(describe_kind(
            SftpErrorKind::Other,
            "rmdir",
            path,
            "refusing to operate on the filesystem root",
        ));
    }

    if normalized
        .split('/')
        .any(|segment| segment == "." || segment == "..")
    {
        return Err(describe_kind(
            SftpErrorKind::Other,
            "rmdir",
            path,
            "refusing path containing dot segments",
        ));
    }

    Ok(())
}

/// Recursively remove a directory and all of its children.
pub async fn rmdir_recursive(sftp: &SftpSession, path: &str) -> Result<(), String> {
    validate_recursive_target(path)?;

    let mut directory_stack = vec![(path.to_string(), 0usize)];
    let mut removal_order = Vec::new();
    // A server that reports a symbolic link as a directory would otherwise
    // keep the walk alive forever.
    let mut visited = std::collections::HashSet::new();

    while let Some((current_dir, depth)) = directory_stack.pop() {
        if depth > MAX_RECURSIVE_DEPTH {
            return Err(describe_kind(
                SftpErrorKind::Other,
                "rmdir",
                &current_dir,
                "directory tree too deep",
            ));
        }
        if !visited.insert(current_dir.clone()) {
            continue;
        }

        removal_order.push(current_dir.clone());
        let entries = list_dir(sftp, &current_dir).await?;
        for entry in entries {
            if entry.is_dir {
                directory_stack.push((entry.path, depth + 1));
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
        .map_err(|e| describe("rename", old_path, &e.to_string()))
}

/// Get file/dir metadata.
pub async fn stat(sftp: &SftpSession, path: &str) -> Result<FileEntry, String> {
    let attrs = sftp
        .metadata(path)
        .await
        .map_err(|e| describe("stat", path, &e.to_string()))?;
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
        .map_err(|e| describe("chmod", path, &e.to_string()))
}

/// Append bytes to an existing file.
pub async fn append_file(sftp: &SftpSession, path: &str, data: &[u8]) -> Result<(), String> {
    use russh_sftp::protocol::OpenFlags;
    use tokio::io::AsyncWriteExt;
    let mut file = sftp
        .open_with_flags(path, OpenFlags::WRITE | OpenFlags::APPEND)
        .await
        .map_err(|e| describe("open", path, &e.to_string()))?;
    file.write_all(data)
        .await
        .map_err(|e| describe("append", path, &e.to_string()))?;
    file.shutdown()
        .await
        .map_err(|e| describe("flush", path, &e.to_string()))?;
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
        .map_err(|e| describe("open", source_path, &e.to_string()))?;
    let mut target = sftp
        .create(target_path)
        .await
        .map_err(|e| describe("create", target_path, &e.to_string()))?;

    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = source
            .read(&mut buffer)
            .await
            .map_err(|e| describe("read", source_path, &e.to_string()))?;
        if read == 0 {
            break;
        }

        target
            .write_all(&buffer[..read])
            .await
            .map_err(|e| describe("write", target_path, &e.to_string()))?;
    }

    target
        .shutdown()
        .await
        .map_err(|e| describe("flush", target_path, &e.to_string()))?;
    Ok(())
}

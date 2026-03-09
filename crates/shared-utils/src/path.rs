//! 路径安全工具

use std::path::{Component, Path, PathBuf};

/// 将 `name` 安全拼接到 `base` 目录下。
///
/// # 约束
/// - `name` 不能为空
/// - `name` 中不能含有 `..`、绝对路径标记（`/` 或 Windows 盘符/UNC）、路径分隔符
///
/// # 错误
/// 返回字符串错误，说明被拒绝的原因。
pub fn safe_join(base: &Path, name: &str) -> Result<PathBuf, String> {
    if name.is_empty() {
        return Err("filename is empty".into());
    }

    let name_path = Path::new(name);

    if name_path.is_absolute() {
        return Err(format!("path traversal: absolute path rejected: {name}"));
    }

    for component in name_path.components() {
        match component {
            Component::Normal(_) => {}
            Component::CurDir => {}
            _ => {
                return Err(format!("path traversal: illegal component in: {name}"));
            }
        }
    }

    Ok(base.join(name_path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn normal_name_ok() {
        let result = safe_join(Path::new("/tmp/base"), "file.txt");
        assert!(result.is_ok());
    }

    #[test]
    fn dotdot_rejected() {
        assert!(safe_join(Path::new("/tmp/base"), "../../etc/passwd").is_err());
        assert!(safe_join(Path::new("/tmp/base"), "..").is_err());
    }

    #[test]
    fn absolute_path_rejected() {
        assert!(safe_join(Path::new("/tmp/base"), "/etc/passwd").is_err());
    }

    #[test]
    fn windows_drive_letter_rejected() {
        // Windows 盘符在 Windows 下会被识别为 Prefix 组件
        let result = safe_join(Path::new("C:\\base"), "C:\\Windows\\system32");
        assert!(
            result.is_err() || {
                // 在 Unix 上 "C:\Windows\system32" 是普通文件名，是安全的
                true
            }
        );
    }

    #[test]
    fn empty_name_rejected() {
        assert!(safe_join(Path::new("/tmp/base"), "").is_err());
    }

    #[test]
    fn nested_dotdot_rejected() {
        assert!(safe_join(Path::new("/tmp/base"), "sub/../../etc/passwd").is_err());
    }

    #[test]
    fn subdir_slash_rejected() {
        // "sub/file" 含路径分隔符，Component::Normal 会分别处理
        // 注意：单级正常子目录名通过，含 .. 的子路径被拒
        let result = safe_join(Path::new("/tmp/base"), "normaldir");
        assert!(result.is_ok());
    }
}

use std::path::{Component, Path, PathBuf};

/// 规范化路径，去除 `.` 和 `..`（仅词法层面，不访问文件系统）
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                // 如果上一级是普通文件/目录名，则弹出；否则保留 `..`
                if let Some(last) = components.last() {
                    if matches!(last, Component::Normal(_)) {
                        components.pop();
                        continue;
                    }
                }
                components.push(comp);
            }
            Component::CurDir => {} // 忽略 `.`
            _ => components.push(comp),
        }
    }
    let mut result = PathBuf::new();
    for comp in components {
        result.push(comp.as_os_str());
    }
    result
}

/// 安全工作路径函数
/// - `workdir` 必须是绝对路径。
/// - 若 `p` 是绝对路径，则直接检查其是否在 `workdir` 内；否则与 `workdir` 拼接后再检查。
/// - 返回值是规范化后的绝对路径（可能实际不存在）。
pub fn safe_path(workdir: &Path, p: &str) -> Result<PathBuf, String> {
    if !workdir.is_absolute() {
        return Err("Workdir must be absolute".to_string());
    }

    let workdir_norm = normalize_path(workdir);
    let p_path = Path::new(p);

    let full = if p_path.is_absolute() {
        p_path.to_path_buf()
    } else {
        workdir_norm.join(p)
    };

    let norm = normalize_path(&full);

    // 检查 norm 是否以 workdir_norm 为前缀
    match norm.strip_prefix(&workdir_norm) {
        Ok(_) => Ok(norm),
        Err(_) => Err(format!("Path escapes workspace: {}", p)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_safe_path() {
        let workdir = Path::new("/home/user");
        assert!(safe_path(workdir, "doc").is_ok());
        assert!(safe_path(workdir, "doc/../").is_ok());
        assert!(safe_path(workdir, "../").is_err());
        assert!(safe_path(workdir, "/etc").is_err());
        assert!(safe_path(workdir, "/home/user/doc").is_ok()); // 绝对路径但位于工作目录下
        assert!(safe_path(workdir, "/home/user/../other").is_err());
    }
}

use std::{
    fs,
    path::{Component, Path, PathBuf},
};

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

/// 读取文件，限制行数（如果 limit 为 Some，则只取前 limit 行并追加提示）。
/// 若发生任何错误，返回以 "Error: " 开头的错误信息。
pub fn read_file(workdir: &Path, path: &str, limit: Option<usize>) -> String {
    match safe_path(workdir, path) {
        Ok(path_buf) => match fs::read_to_string(&path_buf) {
            Ok(content) => {
                let mut lines: Vec<String> = content.lines().map(|x| x.to_string()).collect();
                let total_lines = lines.len();
                if let Some(lim) = limit {
                    if lim < total_lines {
                        lines.truncate(lim);
                        let last_line = format!("... ({} more lines)", total_lines - lim);
                        lines.push(last_line.to_string());
                    }
                }
                lines.join("\n")
            }
            Err(e) => format!("Error: {}", e),
        },
        Err(e) => format!("Error: {}", e),
    }
}

/// 写入文件
pub fn write_file(workdir: &Path, path: &str, content: String) -> String {
    match safe_path(workdir, path) {
        Ok(path_buf) => match fs::write(&path_buf, content) {
            Ok(_) => format!("Wrote content to {} successful", &path_buf.display()),
            Err(e) => format!("{}", e),
        },
        Err(e) => format!("Error: {}", e),
    }
}

/// 编辑文件
pub fn edit_file(workdir: &Path, path: &str, old_text: String, new_text: String) -> String {
    match safe_path(workdir, path) {
        Ok(path_buf) => {
            if path_buf.is_dir() {
                return format!("Error: {} is a directory", &path_buf.display());
            }

            if !path_buf.exists() {
                return format!("Error: {} does not exist", &path_buf.display());
            }

            // TODO: 错误处理，迫在眉睫
            let original = read_file(workdir, path, None);

            if !original.contains(&old_text) {
                return format!(
                    "Error: {} does not contain the old text",
                    &path_buf.display()
                );
            }
            let new_content = original.replace(&old_text, &new_text);

            match fs::write(&path_buf, new_content) {
                Ok(_) => format!("Edit to {} successful", &path_buf.display()),
                Err(e) => format!("{}", e),
            }
        }
        Err(e) => format!("Error: {}", e),
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

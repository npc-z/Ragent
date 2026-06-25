use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::error::RagentError;

/// 规范化路径，去除 `.` 和 `..`（仅词法层面，不访问文件系统）
fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for comp in path.components() {
        match comp {
            Component::ParentDir => {
                // 如果上一级是普通文件/目录名，则弹出；否则保留 `..`
                if let Some(last) = components.last()
                    && matches!(last, Component::Normal(_))
                {
                    components.pop();
                    continue;
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
pub fn read_file(workdir: &Path, path: &str, limit: Option<usize>) -> Result<String, RagentError> {
    let path_buf = safe_path(workdir, path).map_err(RagentError::PathEscape)?;
    let content = fs::read_to_string(&path_buf)?;

    let mut lines = content
        .lines()
        .map(|x| x.to_string())
        .collect::<Vec<String>>();
    let total_lines = lines.len();

    if let Some(lim) = limit
        && lim < total_lines
    {
        lines.truncate(lim);
        lines.push(format!("... ({} more lines)", total_lines - lim));
    }

    Ok(lines.join("\n"))
}

/// 写入文件
pub fn write_file(workdir: &Path, path: &str, content: String) -> Result<String, RagentError> {
    let path_buf = safe_path(workdir, path).map_err(RagentError::PathEscape)?;
    fs::write(&path_buf, content)?;
    Ok(format!(
        "Wrote content to {} successful",
        path_buf.display()
    ))
}

/// 编辑文件
// TODO: TOCTOU 问题,  read_file 和 write_file 之间有竞态窗口
pub fn edit_file(
    workdir: &Path,
    path: &str,
    old_text: String,
    new_text: String,
) -> Result<String, RagentError> {
    let path_buf = safe_path(workdir, path).map_err(RagentError::PathEscape)?;

    if path_buf.is_dir() {
        return Err(RagentError::PathNotAFile(path_buf.display().to_string()));
    }

    if !path_buf.exists() {
        return Err(RagentError::PathNotExist(path_buf.display().to_string()));
    }

    let mut original = read_file(workdir, path, None)?;
    let idx = original
        .find(&old_text)
        .ok_or_else(|| RagentError::TextNotFound(old_text.clone()))?;
    // replace only the first occurrence of old_text with new_text
    original.replace_range(idx..idx + old_text.len(), &new_text);
    write_file(workdir, path, original)?;
    Ok(format!(
        "Replaced first occurrence of '{}' with '{}' in {}",
        old_text,
        new_text,
        path_buf.display()
    ))
}

/// Glob 文件路径
pub fn glob_file(workdir: &Path, path: &str, pattern: String) -> Result<String, RagentError> {
    let path_buf = safe_path(workdir, path).map_err(RagentError::PathEscape)?;

    if !path_buf.exists() {
        return Err(RagentError::PathNotExist(path_buf.display().to_string()));
    }
    let mut matched_files = Vec::new();

    let walker = walkdir::WalkDir::new(&path_buf).into_iter();

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            let file_name = entry.file_name().to_string_lossy();
            if glob::Pattern::new(&pattern)
                .map(|p| p.matches(&file_name))
                .unwrap_or(false)
            {
                matched_files.push(entry.path().display().to_string());
            }
        }
    }

    if matched_files.is_empty() {
        return Ok(format!(
            "No files matched the pattern '{}' in '{}'",
            pattern,
            path_buf.display()
        ));
    }

    Ok(matched_files.join("\n"))
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

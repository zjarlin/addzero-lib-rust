use std::path::PathBuf;

/// 返回用户的 home 目录。
///
/// 优先读取 `HOME` 环境变量；未设置时回退到当前目录 `.`。
#[must_use]
pub fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

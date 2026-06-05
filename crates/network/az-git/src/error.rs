use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, AzGitError>;

#[derive(Debug, Error)]
pub enum AzGitError {
    #[error("配置目录不可用")]
    ConfigDirUnavailable,
    #[error("读取配置 {path} 失败：{source}")]
    ReadConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("写入配置 {path} 失败：{source}")]
    WriteConfig {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("解析配置 {path} 失败：{source}")]
    ParseConfig {
        path: PathBuf,
        source: serde_json::Error,
    },
    #[error("编码配置失败：{0}")]
    EncodeConfig(#[from] serde_json::Error),
    #[error("执行 {program} 失败：{source}")]
    Command {
        program: String,
        source: std::io::Error,
    },
    #[error("命令 {program} 执行失败：{stderr}")]
    CommandFailed { program: String, stderr: String },
    #[error("解析 {program} 输出失败：{source}")]
    ParseCommandOutput {
        program: String,
        source: serde_json::Error,
    },
    #[error("命令 {program} 在 {timeout_ms}ms 后超时")]
    CommandTimeout { program: String, timeout_ms: u64 },
}

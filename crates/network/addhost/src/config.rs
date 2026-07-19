use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

const DEFAULT_REMOTE_PORT_START: u16 = 20_000;
const DEFAULT_REMOTE_PORT_END: u16 = 29_999;

/// 公网中转机与域名配置。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RelayConfig {
    pub server: String,
    pub domain: String,
    pub remote_port_start: u16,
    pub remote_port_end: u16,
}

impl RelayConfig {
    /// 创建并校验一份中转配置。
    pub fn create(server: String, domain: String) -> Result<Self> {
        let config = Self {
            server,
            domain: normalize_host(&domain)?,
            remote_port_start: DEFAULT_REMOTE_PORT_START,
            remote_port_end: DEFAULT_REMOTE_PORT_END,
        };
        config.validate()?;
        Ok(config)
    }

    /// 校验 SSH 目标、域名和远端端口池。
    pub fn validate(&self) -> Result<()> {
        validate_server(&self.server)?;
        normalize_host(&self.domain)?;

        if self.remote_port_start < 1024 {
            bail!("远端端口池起点必须大于等于 1024");
        }
        if self.remote_port_start > self.remote_port_end {
            bail!("远端端口池起点不能大于终点");
        }
        Ok(())
    }
}

/// CLI 使用的配置与状态文件路径。
#[derive(Clone, Debug)]
pub struct StoragePaths {
    pub config_file: PathBuf,
    pub mappings_file: PathBuf,
    pub state_dir: PathBuf,
}

impl StoragePaths {
    /// 按当前操作系统约定定位配置和运行状态目录。
    pub fn discover() -> Result<Self> {
        let config_root = dirs::config_dir().context("无法定位用户配置目录")?;
        let state_root = dirs::state_dir()
            .or_else(dirs::data_local_dir)
            .context("无法定位用户状态目录")?;

        let config_dir = config_root.join("addhost");
        let state_dir = state_root.join("addhost");
        Ok(Self {
            config_file: config_dir.join("config.toml"),
            mappings_file: config_dir.join("mappings.toml"),
            state_dir,
        })
    }
}

/// 读取并校验已初始化的中转配置。
pub fn load_config(paths: &StoragePaths) -> Result<RelayConfig> {
    let source = fs::read_to_string(&paths.config_file).with_context(|| {
        format!(
            "读取配置失败，请先执行 addhost init：{}",
            paths.config_file.display()
        )
    })?;
    let config: RelayConfig = toml_edit::de::from_str(&source)
        .with_context(|| format!("解析配置失败：{}", paths.config_file.display()))?;
    config.validate()?;
    Ok(config)
}

/// 将中转配置写入用户配置目录。
pub fn save_config(paths: &StoragePaths, config: &RelayConfig) -> Result<()> {
    config.validate()?;
    let source = toml_edit::ser::to_string_pretty(config).context("序列化中转配置失败")?;
    write_user_file(&paths.config_file, &source)
}

pub(crate) fn write_user_file(path: &Path, source: &str) -> Result<()> {
    let parent = path
        .parent()
        .with_context(|| format!("文件缺少父目录：{}", path.display()))?;
    fs::create_dir_all(parent).with_context(|| format!("创建目录失败：{}", parent.display()))?;
    fs::write(path, source).with_context(|| format!("写入文件失败：{}", path.display()))
}

/// 规范化并校验一个单级子域名标签。
pub fn normalize_name(value: &str) -> Result<String> {
    let normalized = value.trim().to_ascii_lowercase();
    validate_dns_label(&normalized, "子域名")?;
    Ok(normalized)
}

/// 规范化并校验一个完整域名。
pub fn normalize_host(value: &str) -> Result<String> {
    let normalized = value.trim().trim_end_matches('.').to_ascii_lowercase();
    let labels: Vec<&str> = normalized.split('.').collect();
    if labels.len() < 2 {
        bail!("域名必须是可公开解析的完整域名，例如 dev.example.com");
    }

    for label in labels {
        validate_dns_label(label, "域名")?;
    }
    Ok(normalized)
}

fn validate_dns_label(value: &str, field: &str) -> Result<()> {
    if value.is_empty() || value.len() > 63 {
        bail!("{field}标签长度必须在 1 到 63 之间");
    }

    let starts_or_ends_with_hyphen = value.starts_with('-') || value.ends_with('-');
    if starts_or_ends_with_hyphen {
        bail!("{field}标签不能以连字符开头或结尾");
    }

    if !value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        bail!("{field}标签只能包含小写字母、数字和连字符");
    }
    Ok(())
}

fn validate_server(value: &str) -> Result<()> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.starts_with('-') {
        bail!("SSH 服务器不能为空，也不能以连字符开头");
    }
    if trimmed.bytes().any(|byte| byte.is_ascii_whitespace()) {
        bail!("SSH 服务器不能包含空白字符");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn normalizes_dns_values() -> Result<()> {
        assert_eq!(normalize_name(" Demo-1 ")?, "demo-1");
        assert_eq!(normalize_host("Dev.Example.COM.")?, "dev.example.com");
        Ok(())
    }

    #[test]
    fn rejects_unsafe_names_and_servers() {
        assert!(normalize_name("foo.bar").is_err());
        assert!(normalize_name("-foo").is_err());
        assert!(
            RelayConfig::create("-oProxyCommand=x".to_owned(), "example.com".to_owned()).is_err()
        );
    }
}

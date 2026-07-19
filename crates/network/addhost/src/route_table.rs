use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{normalize_host, write_user_file};

/// 公网 relay 使用的域名到回环端口映射。
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RouteTable {
    pub routes: BTreeMap<String, u16>,
}

impl RouteTable {
    /// 从指定文件读取路由；文件不存在时返回空表。
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::default());
        }

        let source = fs::read_to_string(path)
            .with_context(|| format!("读取 relay 路由失败：{}", path.display()))?;
        toml_edit::de::from_str(&source)
            .with_context(|| format!("解析 relay 路由失败：{}", path.display()))
    }

    /// 保存路由表。
    pub fn save(&self, path: &Path) -> Result<()> {
        let source = toml_edit::ser::to_string_pretty(self).context("序列化 relay 路由失败")?;
        write_user_file(path, &source)
    }

    /// 新增或替换完整域名路由。
    pub fn set(&mut self, host: &str, port: u16) -> Result<String> {
        let host = normalize_host(host)?;
        self.routes.insert(host.clone(), port);
        Ok(host)
    }

    /// 删除完整域名路由。
    pub fn remove(&mut self, host: &str) -> Result<Option<u16>> {
        let host = normalize_host(host)?;
        Ok(self.routes.remove(&host))
    }

    /// 查找请求域名对应的回环端口。
    pub fn resolve(&self, host: &str) -> Option<u16> {
        let normalized = host.trim().trim_end_matches('.').to_ascii_lowercase();
        self.routes.get(&normalized).copied()
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use super::*;

    #[test]
    fn route_lookup_is_case_insensitive() -> Result<()> {
        let mut table = RouteTable::default();
        table.set("Demo.Example.com", 20_001)?;
        assert_eq!(table.resolve("demo.example.com"), Some(20_001));
        assert_eq!(table.resolve("DEMO.EXAMPLE.COM."), Some(20_001));
        Ok(())
    }
}

use std::fs;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::config::{StoragePaths, write_user_file};

/// 一个已发布的本地端口映射。
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct HostMapping {
    pub name: String,
    pub host: String,
    pub local_port: u16,
    pub remote_port: u16,
}

/// 当前用户维护的全部端口映射。
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MappingRegistry {
    pub mappings: Vec<HostMapping>,
}

impl MappingRegistry {
    /// 按子域名查找映射。
    pub fn find(&self, name: &str) -> Option<&HostMapping> {
        self.mappings.iter().find(|mapping| mapping.name == name)
    }

    /// 新增或替换同名映射。
    pub fn upsert(&mut self, mapping: HostMapping) {
        self.mappings.retain(|item| item.name != mapping.name);
        self.mappings.push(mapping);
        self.mappings
            .sort_by(|left, right| left.name.cmp(&right.name));
    }

    /// 删除同名映射并返回原值。
    pub fn remove(&mut self, name: &str) -> Option<HostMapping> {
        let index = self
            .mappings
            .iter()
            .position(|mapping| mapping.name == name)?;
        Some(self.mappings.remove(index))
    }
}

/// 读取映射状态；首次使用时返回空集合。
pub fn load_mappings(paths: &StoragePaths) -> Result<MappingRegistry> {
    if !paths.mappings_file.exists() {
        return Ok(MappingRegistry::default());
    }

    let source = fs::read_to_string(&paths.mappings_file)
        .with_context(|| format!("读取映射状态失败：{}", paths.mappings_file.display()))?;
    toml_edit::de::from_str(&source)
        .with_context(|| format!("解析映射状态失败：{}", paths.mappings_file.display()))
}

/// 保存当前映射状态。
pub fn save_mappings(paths: &StoragePaths, registry: &MappingRegistry) -> Result<()> {
    let source = toml_edit::ser::to_string_pretty(registry).context("序列化映射状态失败")?;
    write_user_file(&paths.mappings_file, &source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upsert_replaces_same_name() {
        let mut registry = MappingRegistry::default();
        registry.upsert(HostMapping {
            name: "demo".to_owned(),
            host: "demo.example.com".to_owned(),
            local_port: 8080,
            remote_port: 20_001,
        });
        registry.upsert(HostMapping {
            name: "demo".to_owned(),
            host: "demo.example.com".to_owned(),
            local_port: 9090,
            remote_port: 20_001,
        });

        assert_eq!(registry.mappings.len(), 1);
        assert_eq!(registry.mappings[0].local_port, 9090);
    }
}

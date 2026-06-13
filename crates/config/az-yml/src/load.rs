//! YAML 文件加载入口。

use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use serde::de::DeserializeOwned;
use serde_yaml::Value;

use crate::path::YamlDoc;

/// 从文件读取 YAML 并反序列化为调用方指定类型。
///
/// 文件读取和 YAML 解析失败会保留底层错误，并附加实际路径上下文。
pub fn load_yaml<T, P>(path: P) -> Result<T>
where
    T: DeserializeOwned,
    P: AsRef<Path>,
{
    let path = path.as_ref();
    let content = fs::read_to_string(path)
        .with_context(|| format!("failed to read yaml file at {}", path.display()))?;
    serde_yaml::from_str::<T>(&content)
        .with_context(|| format!("failed to parse yaml file at {}", path.display()))
}

/// 从文件读取 YAML 并保留为可路径查询的 [`YamlDoc`]。
pub fn load_yaml_value<P>(path: P) -> Result<YamlDoc>
where
    P: AsRef<Path>,
{
    load_yaml::<Value, _>(path).map(YamlDoc::from_value)
}

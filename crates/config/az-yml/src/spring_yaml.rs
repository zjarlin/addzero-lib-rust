//! Spring Boot 风格 YAML 配置目录读取器。

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use az_derive_aliases::{apply, plain_eq};

use crate::load::load_yaml_value;
use crate::path::YamlDoc;

/// Spring Boot 风格 YAML 配置目录读取器。
///
/// 读取边界限定在一个根目录下，资源名会解析为 `*.yml` / `*.yaml`，并支持 `application-{profile}` 激活文件。
#[apply(plain_eq)]
pub struct SpringYaml {
    root: PathBuf,
}

impl SpringYaml {
    /// 指定配置根目录。
    pub fn from_dir(path: impl Into<PathBuf>) -> Self {
        Self { root: path.into() }
    }

    /// 使用当前工作目录作为配置根目录。
    pub fn from_current_dir() -> Result<Self> {
        let root = env::current_dir().context("failed to read current directory")?;
        Ok(Self { root })
    }

    /// 返回配置根目录。
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// 将逻辑资源名解析为实际文件路径。
    ///
    /// 不带扩展名时优先查找 `.yml`，再查找 `.yaml`；找不到文件时返回默认候选路径，不主动创建文件。
    pub fn resolve_resource(&self, resource_name: &str) -> PathBuf {
        let base_name = resource_name
            .strip_suffix(".yml")
            .or_else(|| resource_name.strip_suffix(".yaml"))
            .unwrap_or(resource_name);

        let extensions = if resource_name.contains('.') {
            ["", ".yml", ".yaml"]
        } else {
            [".yml", ".yaml", ""]
        };

        for extension in extensions {
            if extension.is_empty() && !resource_name.contains('.') {
                continue;
            }

            let candidate = self.root.join(format!("{base_name}{extension}"));
            if candidate.exists() {
                return candidate;
            }
        }

        let fallback_extension = if resource_name.contains('.') {
            ""
        } else {
            ".yml"
        };
        self.root.join(format!("{base_name}{fallback_extension}"))
    }

    /// 读取指定 YAML 资源的原始文本内容。
    pub fn get_yml_content(&self, resource_name: &str) -> Result<String> {
        let path = self.resolve_resource(resource_name);
        fs::read_to_string(&path)
            .with_context(|| format!("failed to read yaml file at {}", path.display()))
    }

    /// 读取指定 YAML 资源并解析为 [`YamlDoc`]。
    pub fn load_named(&self, resource_name: &str) -> Result<YamlDoc> {
        let path = self.resolve_resource(resource_name);
        load_yaml_value(path)
    }

    /// 读取当前激活的 Spring Boot 配置。
    ///
    /// 先读取 `application.yml` / `application.yaml`，再根据 `spring.profiles.active` 尝试加载
    /// `application-{profile}.yml` 或 `.yaml`。若 profile 文件不存在，则回退到主配置文档。
    pub fn load_active(&self) -> Result<YamlDoc> {
        let primary = self.load_named("application")?;
        let profile = primary
            .get_string("spring.profiles.active")?
            .filter(|value| !value.trim().is_empty());

        if let Some(profile_name) = profile {
            let active_path = self.resolve_resource(&format!("application-{profile_name}"));
            if active_path.exists() {
                return load_yaml_value(active_path);
            }
        }

        Ok(primary)
    }
}

//! 从 Spring YAML 中读取数据库连接配置。

use std::path::Path;

use anyhow::Result;

use crate::path::YamlDoc;
use crate::spring_yaml::SpringYaml;

/// 从 Spring YAML 中提取出的数据库连接配置。
///
/// `Debug` 输出会自动隐藏 `jdbc_password`，避免日志中泄漏明文密码。
#[derive(Clone, derive_more::Debug, Eq, PartialEq)]
pub struct DatabaseConfig {
    /// JDBC 或 R2DBC 连接 URL。
    pub jdbc_url: String,
    /// 数据库用户名；配置缺失或为空白时为 `None`。
    pub jdbc_username: Option<String>,
    /// 数据库密码；配置缺失或为空白时为 `None`，调试输出会脱敏。
    #[debug(skip)]
    pub jdbc_password: Option<String>,
}

/// Spring YAML 数据库连接配置读取器。
///
/// 支持常见单数据源路径、`master` / `primary` 等命名数据源，以及调用方指定的优先数据源名。
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DatabaseConfigReader;

impl DatabaseConfigReader {
    /// 从配置目录读取激活配置并尝试提取数据库连接信息。
    ///
    /// 返回 `Ok(None)` 表示 YAML 可读取但没有发现受支持的数据源路径。
    pub fn read(
        path: impl AsRef<Path>,
        prefer_data_source_name: Option<&str>,
    ) -> Result<Option<DatabaseConfig>> {
        let spring_yaml = SpringYaml::from_dir(path.as_ref().to_path_buf());
        let active = spring_yaml.load_active()?;
        Self::read_from_doc(&active, prefer_data_source_name)
    }

    /// 从已经加载的 YAML 文档中提取数据库连接信息。
    ///
    /// 当指定 `prefer_data_source_name` 时会先查找该命名数据源；未命中时再按内置单数据源和常见命名数据源顺序回退。
    pub fn read_from_doc(
        doc: &YamlDoc,
        prefer_data_source_name: Option<&str>,
    ) -> Result<Option<DatabaseConfig>> {
        if let Some(name) = prefer_data_source_name
            && let Some(config) = Self::read_named_data_source(doc, name)?
        {
            return Ok(Some(config));
        }

        for url_path in SINGLE_DATASOURCE_PATHS {
            if let Some(url) = read_non_blank_property(doc, url_path)? {
                let base_path = extract_base_path(url_path);
                let username = read_non_blank_property(doc, &format!("{base_path}.username"))?
                    .or(read_non_blank_property(doc, "spring.datasource.username")?)
                    .or(read_non_blank_property(doc, "spring.r2dbc.username")?);
                let password = read_non_blank_property(doc, &format!("{base_path}.password"))?
                    .or(read_non_blank_property(doc, "spring.datasource.password")?)
                    .or(read_non_blank_property(doc, "spring.r2dbc.password")?);

                return Ok(Some(DatabaseConfig {
                    jdbc_url: url,
                    jdbc_username: username,
                    jdbc_password: password,
                }));
            }
        }

        for data_source_name in COMMON_DATA_SOURCE_NAMES {
            if let Some(config) = Self::read_named_data_source(doc, data_source_name)? {
                return Ok(Some(config));
            }
        }

        Ok(None)
    }

    fn read_named_data_source(
        doc: &YamlDoc,
        data_source_name: &str,
    ) -> Result<Option<DatabaseConfig>> {
        let url_paths = [
            format!("spring.datasource.{data_source_name}.url"),
            format!("spring.datasource.{data_source_name}.jdbc-url"),
            format!("spring.datasource.dynamic.datasource.{data_source_name}.url"),
            format!("spring.datasource.mp.datasource.{data_source_name}.url"),
        ];

        for url_path in url_paths {
            if let Some(url) = read_non_blank_property(doc, &url_path)? {
                let base_path = extract_base_path(&url_path);
                let username = read_non_blank_property(doc, &format!("{base_path}.username"))?;
                let password = read_non_blank_property(doc, &format!("{base_path}.password"))?;

                return Ok(Some(DatabaseConfig {
                    jdbc_url: url,
                    jdbc_username: username,
                    jdbc_password: password,
                }));
            }
        }

        Ok(None)
    }
}

const SINGLE_DATASOURCE_PATHS: &[&str] = &[
    "spring.datasource.url",
    "spring.datasource.jdbc-url",
    "spring.r2dbc.url",
    "spring.datasource.primary.url",
    "spring.datasource.master.url",
    "spring.datasource.default.url",
    "spring.data.jdbc.url",
];

const COMMON_DATA_SOURCE_NAMES: &[&str] = &["master", "primary", "default", "main", "slave"];

fn read_non_blank_property(doc: &YamlDoc, path: &str) -> Result<Option<String>> {
    Ok(doc
        .get_string(path)?
        .filter(|value| !value.trim().is_empty()))
}

fn extract_base_path(url_path: &str) -> String {
    url_path
        .rsplit_once('.')
        .map(|(base, _)| {
            base.trim_end_matches(".jdbc")
                .trim_end_matches(".r2dbc")
                .to_owned()
        })
        .unwrap_or_else(|| url_path.to_owned())
}

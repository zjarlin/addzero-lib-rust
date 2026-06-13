//! # az-yml
//!
//! YAML 配置文件的加载、路径查询与值提取工具库，面向 Spring Boot 风格配置约定。
//!
//! 主要功能：
//!
//! - [`load::load_yaml`] / [`load::load_yaml_value`] — 从文件读取并解析 YAML。
//! - [`path::YamlPath`] — 点号路径解析，支持 `a.b[0]`、带引号的键名等嵌套访问语法。
//! - [`path::YamlDoc`] — 封装 `serde_yaml::Value`，提供路径查询方法。
//! - [`path::YamlLookup`] — 通用路径查找 trait。
//! - [`spring_yaml::SpringYaml`] — Spring Boot 风格配置 profile 激活读取。
//! - [`database_config_reader::DatabaseConfigReader`] — 扫描 JDBC / R2DBC 数据源配置。
//!
//! 宏：
//!
//! - [`yaml_path!`] — 确定性路径解析，非法路径 panic。
//! - [`yaml_get!`] — 组合路径解析与值查询。

automod::dir!(pub "src");

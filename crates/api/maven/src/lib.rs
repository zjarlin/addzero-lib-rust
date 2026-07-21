//! Maven Central 搜索 API 客户端——查询、版本检索与制品下载。
//!
//! 本 crate 封装了 [search.maven.org](https://search.maven.org) 的 REST API，
//! 提供按 `groupId`、`artifactId`、全限定类名、SHA1、关键词等多种维度的搜索能力，
//! 并支持直接下载制品文件。
//!
//! ## 快速开始
//!
//! ```rust
//! use az_maven::maven::create_maven_central_api;
//!
//! # fn main() -> anyhow::Result<()> {
//! let api = create_maven_central_api()?;
//! let artifacts = api.search_by_keyword("serde", 10)?;
//! for a in &artifacts {
//!     println!("{}: {} v{}", a.group_id, a.artifact_id,
//!         a.resolved_version().unwrap_or("unknown"));
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## 模块结构
//!
//! - [`MavenCentralApi`] — 主要查询入口
//! - [`MavenArtifact`] — 制品结构体
//! - [`ApiConfig`] / [`ApiConfigBuilder`] — HTTP 客户端配置（re-export 自 `az-music`）
//! - 内部模块 `http`、`config`、`util` 封装 HTTP 调用、签名和工具函数

#![forbid(unsafe_code)]

automod::dir!(pub "src");

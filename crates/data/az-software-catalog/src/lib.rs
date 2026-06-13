//! 软件目录管理服务，维护软件条目、安装方式和平台元数据的完整生命周期。
//!
//! 本 crate 为 admin 后台和 CLI 提供统一的软件目录数据模型与持久化能力，
//! 支持 macOS / Windows / Linux 三大平台以及 Brew、winget、Scoop 等多种安装方式。
//!
//! ## 核心能力
//!
//! - **软件条目管理**：通过 [`model::SoftwareEntryDto`] 描述软件的基本信息（名称、厂商、官网、图标等）。
//! - **多平台安装方式**：每条软件可关联多种 [`model::SoftwareInstallMethodDto`]，
//!   涵盖 Homebrew、winget、Scoop、Chocolatey、curl 下载等安装渠道。
//! - **元数据抓取**：从官网 URL 自动提取软件元数据（标题、描述、图标）。
//! - **平台感知**：[`model::current_platform()`] 根据编译目标自动识别当前操作系统。
//! - **WASM 兼容**：核心模型层（`model` 模块）可在 `wasm32` 目标下使用，
//!   仅实体层和持久化层限定为 native 构建。
//!
//! ## 关键类型
//!
//! - [`model::SoftwareCatalogDto`] — 按平台分组的软件目录快照。
//! - [`model::SoftwareEntryDto`] — 单个软件条目，包含标签和安装方式列表。
//! - [`model::SoftwarePlatform`] — 平台枚举（macOS / Windows / Linux）。
//! - [`model::InstallerKind`] — 安装方式枚举（Brew / winget / Scoop 等）。
//! - [`service::SoftwareCatalogService`] — 服务层门面（仅 native 可用）。

automod::dir!(pub "src");

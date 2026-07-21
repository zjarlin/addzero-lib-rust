//! 资产（Asset）图谱管理服务。
//!
//! 提供资产 CRUD、有向边关系图谱、AI Provider 配置与密钥加密存储等功能。
//! 支持 PostgreSQL 持久化和纯内存双模式，连接失败时自动降级到内存模式。
//!
//! # 核心类型
//!
//! - [`service::AssetService`] — 主服务入口，封装所有读写操作
//! - [`types::Asset`] / [`types::AssetEdge`] — 资产节点与边的数据模型
//! - [`types::AiModelProvider`] — AI 服务提供商配置
//! - [`secret::SecretCipher`] — 基于 AES-256-GCM 的 API 密钥加解密

automod::dir!(pub "src");

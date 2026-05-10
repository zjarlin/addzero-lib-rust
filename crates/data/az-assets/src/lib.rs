//! 资产（Asset）图谱管理服务。
//!
//! 提供资产 CRUD、有向边关系图谱、AI Provider 配置与密钥加密存储等功能。
//! 支持 PostgreSQL 持久化和纯内存双模式，连接失败时自动降级到内存模式。
//!
//! # 核心类型
//!
//! - [`AssetService`] — 主服务入口，封装所有读写操作
//! - [`Asset`] / [`AssetEdge`] — 资产节点与边的数据模型
//! - [`AiModelProvider`] — AI 服务提供商配置
//! - [`SecretCipher`] — 基于 AES-256-GCM 的 API 密钥加解密

mod entity;
mod pg_repo;
mod secret;
mod service;
mod types;

pub use pg_repo::PgRepo;
pub use secret::{EncryptedSecret, SecretCipher};
pub use service::AssetService;
pub use types::{
    AiModelProvider, AiModelProviderUpsert, AiPromptButton, AiPromptButtonUpsert, AiProviderKind,
    Asset, AssetEdge, AssetEdgeUpsert, AssetGraph, AssetKind, AssetProviderSecret, AssetUpsert,
    PromptRunOutput, SuggestedEdge,
};

# az-assets

资产（Asset）图谱管理服务，支持 PostgreSQL 持久化与纯内存双模式，内置 AI Provider 密钥加密存储。

## 功能

- 资产 CRUD：支持 `Capture`、`Note`、`Skill`、`Software`、`Package` 五种资产类型
- 资产边（Edge）管理：建立资产间的有向关系图谱，记录关系类型和置信度
- AI Provider 管理：配置 OpenAI / Anthropic / Gemini 等 AI 服务提供商及其 API 密钥
- AI Prompt Button：可配置的 AI 提示按钮，按资产类型绑定提示模板和模型
- 密钥加密：基于 AES-256-GCM 的 API 密钥加密存储，通过主密钥派生
- 双存储模式：PostgreSQL 在线模式与纯内存降级模式，连接失败时自动降级

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-assets = { path = "../assets" }       # workspace 内部引用
# 或发布后：
# az-assets = "0.1"                      # crates.io 引用
```

## 用法

```rust
use az_assets::service::AssetService;
use az_assets::types::{AssetKind, AssetUpsert};

// 内存模式启动
let service = AssetService::memory_only(None);

// 创建资产
let asset = service.upsert_asset(AssetUpsert {
    id: None,
    kind: AssetKind::Note,
    title: "Rust 所有权笔记".into(),
    body: "所有权是 Rust 的核心概念……".into(),
    tags: vec!["rust".into(), "ownership".into()],
    status: "active".into(),
    metadata: serde_json::json!({}),
}).await.unwrap();

// 查询资产列表
let assets = service.list_assets(Some(AssetKind::Note)).await.unwrap();

// 查询资产图谱
let graph = service.graph().await.unwrap();
```

## 依赖的 crates

- `az-persistence` - 数据库连接与持久化上下文
- `sea-orm` / `sqlx` - PostgreSQL 数据库驱动
- `ring` - AES-256-GCM 加密原语
- `chrono` / `uuid` / `serde` - 通用数据类型

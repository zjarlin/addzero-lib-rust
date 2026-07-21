# az-software-catalog

软件目录管理服务，维护软件条目、安装方式和平台元数据的完整生命周期，为 admin 后台和 CLI 提供统一数据模型与持久化能力。

## 功能

- **软件条目管理**：完整的 CRUD 操作，包含名称、厂商、官网、图标、标签等元数据
- **多平台支持**：覆盖 macOS、Windows、Linux 三大平台，`current_platform()` 自动识别编译目标
- **多种安装方式**：支持 Homebrew、winget、Scoop、Chocolatey、curl 下载、直接安装包等 8 种安装渠道
- **元数据抓取**：从官网 URL 自动提取软件标题、描述和图标
- **草稿生成**：`SoftwareDraftInput` 从官网 URL 快速生成软件条目草稿
- **WASM 兼容**：核心模型层可在 `wasm32` 目标下使用，持久化层限定为 native 构建

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-software-catalog = { path = "../software-catalog" }       # workspace 内部引用
# 或发布后：
# az-software-catalog = "0.1"                                   # crates.io 引用
```

## 用法

```rust
use az_software_catalog::model::{
    InstallerKind, SoftwareEntryInput, SoftwareInstallMethodDto, SoftwarePlatform,
    current_platform,
};
use az_software_catalog::service::SoftwareCatalogService;

// 创建软件条目输入
let input = SoftwareEntryInput {
    slug: "cursor".into(),
    title: "Cursor".into(),
    vendor: "Anysphere".into(),
    summary: "AI 原生代码编辑器".into(),
    homepage_url: "https://cursor.com".into(),
    icon_url: "https://cdn.simpleicons.org/cursor".into(),
    trial_platforms: vec![SoftwarePlatform::Macos, SoftwarePlatform::Windows],
    tags: vec!["ide".into(), "agent".into()],
    methods: vec![SoftwareInstallMethodDto {
        id: String::new(),
        platform: SoftwarePlatform::Macos,
        kind: InstallerKind::Brew,
        label: "Homebrew".into(),
        package_id: "cursor".into(),
        asset_item_id: None,
        command: "brew install cursor".into(),
        note: String::new(),
    }],
    ..Default::default()
};
```

## 依赖的 crates

- `az-persistence` — 数据库连接与迁移基础设施（仅 native）
- `toasty` / `reqwest` — PostgreSQL 模型 DSL 和 HTTP 客户端（仅 native）
- `serde` / `serde_json` — 数据序列化
- `uuid` — 软件条目唯一标识
- `anyhow` — 错误返回与上下文
- `regex` — 输入校验（仅 native）
- `chrono` — 时间戳（仅 native）

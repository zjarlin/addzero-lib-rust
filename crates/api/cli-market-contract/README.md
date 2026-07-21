# az-cli-market-contract

CLI 市场数据契约层，定义 CLI 生态中工具条目、安装方式、导入导出等全部共享类型。

## 功能

- CLI 工具市场条目（`CliMarketEntry`）及创建/更新 DTO（`CliMarketEntryUpsert`）
- 完整目录快照（`CliMarketCatalog`）与摘要统计（`CliMarketSummary`）
- JSON / Excel 导入请求与逐行报告（`CliMarketImportRequest` / `CliMarketImportReport`）
- 导出请求与 base64 编码附件（`CliMarketExportRequest` / `CliMarketExportArtifact`）
- 安装执行与结果记录（`CliMarketInstallRequest` / `CliMarketInstallResult`）
- 枚举类型覆盖：生命周期状态、条目类型、语言区域、目标平台、安装方式、导入格式

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-cli-market-contract = { path = "../cli-market-contract" }  # workspace 内部引用
# 或发布后：
# az-cli-market-contract = "0.1"                                  # crates.io 引用
```

## 用法

```rust
use az_cli_market_contract::contract::{
    CliEntryKind, CliImportFormat, CliImportMode, CliMarketEntry, CliMarketImportRequest,
    CliMarketSourceType, CliMarketStatus,
};

// 构造一条市场条目
let entry = CliMarketEntry {
    id: "abc123".into(),
    slug: "my-cli-tool".into(),
    status: CliMarketStatus::Published,
    source_type: CliMarketSourceType::Manual,
    entry_kind: CliEntryKind::Cli,
    ..Default::default()
};

// 构造导入请求（payload 以 base64 编码）
let request = CliMarketImportRequest {
    format: CliImportFormat::Json,
    mode: CliImportMode::Native,
    file_name: "tools.json".into(),
    payload_base64: "eyJ0ZXN0IjoxfQ==".into(),
    submitted_by: "admin".into(),
};
let bytes = request.decode_payload().expect("base64 解码失败");
```

## 依赖的 crates

- `base64` — payload 字段的 base64 编解码
- `serde` / `serde_json` — 所有结构体的序列化与反序列化
- `anyhow` — 错误返回与上下文
- `uuid` — 条目 ID 字段

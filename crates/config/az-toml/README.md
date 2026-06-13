# az-toml

Gradle 风格 Version Catalog TOML 文件（`libs.versions.toml`）的解析、构建、序列化与合并工具库。

## 功能

- 从字符串或文件路径解析 `libs.versions.toml` 为 `VersionCatalog` 结构体
- `load_or_init` — 文件不存在时自动创建默认模板并加载
- `to_string_pretty` — 格式化输出规范的 TOML 文本
- `merge_many` — 合并多个 catalog（版本、插件、bundle 首次优先，库按 group+name 去重）
- `catalog!` 宏 — 以声明式 DSL 内联构建 catalog 值
- `insert_after_table` — 在 TOML 文本中按表名定位并插入内容
- 支持 `version` 直接值和 `version.ref` 引用两种版本选择方式

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-toml = { path = "../az-toml" }      # workspace 内部引用
# 或发布后：
# az-toml = "0.1"                      # crates.io 引用
```

## 用法

```rust
use az_toml::model::VersionCatalog;

// 从文件加载（不存在则创建默认模板）
let catalog = VersionCatalog::load_or_init("libs.versions.toml")?;

// 从字符串解析
let catalog = VersionCatalog::from_str(r#"
[versions]
kotlin = "2.1.0"

[libraries]
hutool = { group = "cn.hutool", name = "hutool-all", version.ref = "kotlin" }
"#)?;

// 序列化为格式化 TOML
let output = catalog.to_string_pretty();

// 合并多个 catalog
let merged = VersionCatalog::merge_many(vec![catalog_a, catalog_b]);

// 使用宏构建
let catalog = az_toml::catalog! {
    versions { kotlin = "2.1.0" }
    libraries { hutool = { group: "cn.hutool", name: "hutool-all", version_ref: "kotlin" } }
    plugins { kotlin = { id: "org.jetbrains.kotlin.jvm", version_ref: "kotlin" } }
    bundles { spring = ["spring-boot", "spring-core"] }
};
```

## 依赖的 crates

- `serde` — 序列化/反序列化框架
- `anyhow` — 错误上下文与统一 `Result`
- `toml_edit` — 保格式的 TOML 读写

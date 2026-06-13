# az-maven

Maven Central 搜索 API 客户端，支持多种维度查询、版本检索与制品文件下载。

## 功能

- 按 `groupId`、`artifactId`、坐标组合、全限定类名、SHA1、标签、关键词搜索制品
- 查询全部版本（`search_all_versions`）
- 获取最新版本号（`get_latest_version`）
- 下载制品文件（`download_file`）
- 可配置的 HTTP 客户端（超时、User-Agent、默认请求头）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-maven = { path = "../az-maven" }  # workspace 内部引用
# 或发布后：
# az-maven = "0.1"                   # crates.io 引用
```

## 用法

```rust
use az_maven::config::ApiConfig;
use az_maven::maven::{MavenCentralApi, create_maven_central_api};

# fn main() -> anyhow::Result<()> {

// 使用默认配置快速创建客户端
let api = create_maven_central_api()?;

// 按关键词搜索
let results = api.search_by_keyword("serde", 10)?;
for artifact in &results {
    println!(
        "{}:{} v{}",
        artifact.group_id,
        artifact.artifact_id,
        artifact.resolved_version().unwrap_or("unknown"),
    );
}

// 获取最新版本号
let version = api.get_latest_version("com.google.guava", "guava")?;
println!("guava 最新版本: {:?}", version);

// 自定义配置
let config = ApiConfig::builder("https://search.maven.org")
    .connect_timeout_secs(5)
    .build()?;
let api = MavenCentralApi::new(config)?;
# Ok(())
# }
```

## 依赖的 crates

- `az-music` — 复用 HTTP 客户端配置基类（`ApiConfig` / `ApiConfigBuilder`）
- `reqwest` — HTTP 同步客户端
- `serde` / `serde_json` — API 响应反序列化
- `sha2` — SHA256 哈希工具函数
- `anyhow` — 统一错误上下文

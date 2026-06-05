# addzero-lib-rust

`addzero-lib-rust` 是一组偏工具型的 Rust workspace，目标是把常用、可复用、低耦合的能力沉淀成独立 crate。

## Codex Desktop

当前应用侧优先维护 `apps/codex/desktop`，它是一个 Dioxus 桌面端壳子，用本地插件系统组装菜单、页面、技能、命令行和环境变量管理能力。

运行入口：

```bash
cargo run --manifest-path apps/codex/desktop/Cargo.toml
```

命令行和环境变量由 Codex 桌面端可视化管理器维护，部署目标仍是 `~/.add_fn`。不要手工编辑 `~/.add_fn`，应通过桌面端或专用部署入口写入：

```bash
cargo run --manifest-path apps/codex/desktop/Cargo.toml -- --deploy-shell-manager
```

Codex 桌面端的本机配置目录：

- `~/.config/addzero/codex/codex.env`
- `~/Library/Application Support/addzero/codex`

PostgreSQL 连接优先读取 `CODEX_DATABASE_URL` 或 `~/.config/addzero/codex/codex.env`，并自动使用 `rs-aio` 作为 Codex 本地数据库名。

## 当前重点模块

| Crate | 说明 |
| --- | --- |
| `az-creates` | 常见 HTTP API 创建器，现已包含 Maven Central、mail.tm、网易云音乐搜索、Suno、天眼查 |
| `az-music` | 独立音乐领域 crate，承载网易云搜索 / 歌词 / Suno 能力 |
| `az-curl` | curl 命令解析、请求构建与响应辅助 |
| `az-email` | SMTP 邮件发送与附件处理 |
| `az-rustfs` | Rust S3 兼容对象存储客户端 |
| `az-minio` | 基于 `az-rustfs` 的 MinIO 便利封装 |
| `az-mqtt` | MQTT blocking 客户端与消息辅助 |
| `az-ssh` | SSH 命令执行与文件传输 |
| `az-excel` | 纯 Rust `.xlsx` 读写与结构处理 |

## 领域目录

- `crates/api/*`：对外 API 聚合与兼容层
- `crates/music/*`：音乐领域能力
- `crates/storage/*`：对象存储与 MinIO 封装
- `crates/network/*`：网络协议与请求辅助
- `crates/config/*`：配置格式处理
- `crates/text/*`：文本处理
- `crates/data/*`：区域与表格数据能力
- `crates/core/*`：基础通用能力
- `crates/runtime/*`：运行时与系统集成

## 快速开始

克隆后直接执行：

```bash
cargo test
```

如果你只想验证新增的 API 创建器：

```bash
cargo test -p az-creates
```

## `az-creates` 用法

### 1. Maven Central

```rust
use az_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::maven_central()?;
    let latest = api.get_latest_version("com.google.guava", "guava")?;
    println!("latest guava version: {latest:?}");
    Ok(())
}
```

### 2. 网易云音乐搜索

```rust
use az_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::music_search()?;
    let songs = api.search_songs("晴天", 5, 0)?;
    println!("first song: {:?}", songs.first().map(|item| &item.name));
    Ok(())
}
```

### 3. Suno

```rust
use az_creates::{Creates, SunoMusicRequest};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::suno("your-suno-token")?;
    let task_id = api.generate_music(&SunoMusicRequest {
        prompt: "写一首城市夜景风格的中文流行歌".to_owned(),
        title: Some("城市夜色".to_owned()),
        tags: Some("pop, chinese".to_owned()),
        ..Default::default()
    })?;
    println!("task id: {task_id}");
    Ok(())
}
```

### 4. 天眼查

```rust
use az_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::tianyancha("your-authorization", "your-x-auth-token")?;
    let result = api.search_company("河南中洛佳科技有限公司", 1, 10, "0")?;
    println!("company count: {:?}", result.company_total);
    Ok(())
}
```

### 5. Temp Mail

```rust
use az_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::temp_mail()?;
    let mailbox = api.create_mailbox_and_login("demo", 12)?;
    println!("temp mailbox: {}", mailbox.address);
    Ok(())
}
```

### 6. 自定义客户端配置

```rust
use std::time::Duration;
use az_creates::{ApiConfig, Creates};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = ApiConfig::builder("https://search.maven.org")
        .connect_timeout(Duration::from_secs(5))
        .request_timeout(Duration::from_secs(15))
        .user_agent("demo-app/0.1.0")
        .build()?;

    let api = Creates::maven_central_with_config(config)?;
    let result = api.search_by_keyword("guava", 5)?;
    println!("hits: {}", result.len());
    Ok(())
}
```

更完整的 `az-creates` API 和范围说明见：

- [crates/api/az-creates/](crates/api/az-creates/)

## 为什么没有直接全量搬运 JVM `network-call`

`addzero-lib-jvm` 的 `network-call` 目录里混着三类东西：

- 通用可公开能力
- 私有供应商接入
- 实验性或站点抓取型实现

这次只迁移了前一类里最适合先落 Rust 版本的部分，避免把不可复用、不可公开、维护成本高的实现包装成“统一 API”。

## 小鳄鱼文档

仓库现在已经补齐 `xiaoeyu.config.json` 和 README 收录规则，后续可以直接用小鳄鱼把 root README、crate README 和 app README 生成成站点文档。

文档接入说明见：

- [docs/README.md](https://github.com/zjarlin/addzero-lib-rust/blob/main/docs/README.md)

## 仓库文档范围

小鳄鱼当前会收录：

- 根目录 `README.md`
- `crates/**/README.md`
- `apps/**/README.md`

这次新增的 `AIO Drive` Git Pool、Finder、同步队列与冲突处理说明，也会跟着这些 README 一起被小鳄鱼站点收录。

默认不会收录：

- `docs/**`
- `target/**`
- 未来你明确标记为内部或实验用途的 README

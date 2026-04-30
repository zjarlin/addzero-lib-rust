# addzero-lib-rust

`addzero-lib-rust` 是一组偏工具型的 Rust workspace，目标是把常用、可复用、低耦合的能力沉淀成独立 crate。

## `msc-aio` 蓝图

当前仓库的一个明确方向是沉淀 `msc-aio`：

- 一套面向个人知识资产、同步任务和命令化能力的 AIO 平台
- 一个以 PostgreSQL 为唯一正式持久化源的后台系统
- 一个同时提供 Dioxus 管理界面、Axum REST API、以及同源 CLI 的工作台

这套系统的目标不是只做一个“网页后台”，而是把自己日常要管理的知识库、同步任务、脚本能力、导入导出流程统一收进同一个平台里，让“万事万物都 CLI 化”成为默认交付形态，而不是补充能力。

### 核心原则

- `all in pg`：正式业务数据全部进入 PostgreSQL，数据库名为 `msc_aio`
- `axum + dioxus`：Axum 负责后端 API 与任务入口，Dioxus 负责管理界面
- `REST + CLI 同源`：后端写 REST API 的同时，CLI 从同一套操作定义生成，避免手写两套接口
- `import != source of truth`：文件系统扫描、构建期嵌入、临时内存实现都只作为导入态或开发态，不作为最终数据落点
- `大功能一模块`：按功能边界拆模块，文件粒度保持在人类可以轻松阅读的范围内

### 规划中的系统分层

1. PostgreSQL `msc_aio`
   作为知识资产、同步任务、操作日志、配置、索引元数据的唯一正式存储
2. 领域服务层
   放在 workspace crate 中，承载知识库、同步、任务、导入导出、资产管理等核心用例
3. Axum API 层
   暴露 REST API、认证、任务调度入口、OpenAPI 文档以及自动化调用面
4. CLI 层
   从与 REST 同源的操作定义生成命令，保证后台能力天然可脚本化
5. Dioxus Admin 层
   作为管理工作台，用于查看知识资产、同步状态、任务历史、配置与系统上下文

### 当前状态

- `msc-aio` 已经开始承接 admin 壳子、多模块场景与知识库可视化
- 技能数据仍有内存实现
- 知识库已经新增 `addzero-knowledge` 数据域 crate，可把本机候选知识目录同步进 PostgreSQL `msc_aio`
- `msc-aio` 的知识页现在会优先从 PG 镜像生成目录，PG 不可用时才退回文件系统快照
- PG 能力在部分底层 crate 已经存在，但还没有完全打通到 `all in pg`
- `msc-aio` 的正式 REST + CLI 同源链路仍处于蓝图阶段，下一步需要优先收敛 contract 与数据模型

### 知识同步落地

当前知识导入先覆盖这台机器上最像“个人知识资产”的来源：

- `~/Desktop/tech-content-automation/rust/sources`
- `~/Nextcloud/一些未整理的资料/Documents`
- `~/Nextcloud/软件文档`
- `~/Nextcloud/DockerCompose`
- `~/Nextcloud/DockerCompose_Unuse`
- `~/memory`
- `~/.config/shell`
- `~/.config/mole`
- `~/Nextcloud/要件`
- `~/Music/addzero/config-sys`

明显的第三方噪音会在导入时跳过，例如 `target/`、`legal/`、`data/`、`LICENSE*`、`CHANGELOG*`、`SECURITY*` 一类文件。

本地运行时，`msc-aio` 与同步 CLI 会按以下优先级读取数据库连接：

1. `MSC_AIO_DATABASE_URL`
2. `DATABASE_URL`
3. `~/.config/msc-aio/msc-aio.env`

示例：

```bash
printf '%s\n' 'MSC_AIO_DATABASE_URL=postgresql://postgres:***@127.0.0.1:15432/msc_aio' > ~/.config/msc-aio/msc-aio.env
cargo run -p addzero-knowledge --bin knowledge-sync
```

更完整的蓝图说明见：

- [docs/plans/2026-04-28-msc-aio-blueprint.md](docs/plans/2026-04-28-msc-aio-blueprint.md)

这次新增了 `addzero-creates`，用于把 `addzero-lib-jvm/lib/tool-jvm/network-call` 里适合公开沉淀的常见 API 收口为 Rust 创建器；同时把音乐领域能力单独抽到了 `addzero-music`。

## 当前重点模块

| Crate | 说明 |
| --- | --- |
| `addzero-creates` | 常见 HTTP API 创建器，现已包含 Maven Central、mail.tm、网易云音乐搜索、Suno、天眼查 |
| `addzero-music` | 独立音乐领域 crate，承载网易云搜索 / 歌词 / Suno 能力 |
| `addzero-curl` | curl 命令解析、请求构建与响应辅助 |
| `addzero-email` | SMTP 邮件发送与附件处理 |
| `addzero-rustfs` | Rust S3 兼容对象存储客户端 |
| `addzero-minio` | 基于 `addzero-rustfs` 的 MinIO 便利封装 |
| `addzero-mqtt` | MQTT blocking 客户端与消息辅助 |
| `addzero-ssh` | SSH 命令执行与文件传输 |
| `addzero-excel` | 纯 Rust `.xlsx` 读写与结构处理 |

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
cargo test -p addzero-creates
```

## `addzero-creates` 用法

### 1. Maven Central

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::maven_central()?;
    let latest = api.get_latest_version("com.google.guava", "guava")?;
    println!("latest guava version: {latest:?}");
    Ok(())
}
```

### 2. 网易云音乐搜索

```rust
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::music_search()?;
    let songs = api.search_songs("晴天", 5, 0)?;
    println!("first song: {:?}", songs.first().map(|item| &item.name));
    Ok(())
}
```

### 3. Suno

```rust
use addzero_creates::{Creates, SunoMusicRequest};

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
use addzero_creates::Creates;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api = Creates::tianyancha("your-authorization", "your-x-auth-token")?;
    let result = api.search_company("河南中洛佳科技有限公司", 1, 10, "0")?;
    println!("company count: {:?}", result.company_total);
    Ok(())
}
```

### 5. Temp Mail

```rust
use addzero_creates::Creates;

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
use addzero_creates::{ApiConfig, Creates};

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

更完整的 `addzero-creates` API 和范围说明见：

- [crates/api/addzero-creates/](crates/api/addzero-creates/)

## 为什么没有直接全量搬运 JVM `network-call`

`addzero-lib-jvm` 的 `network-call` 目录里混着三类东西：

- 通用可公开能力
- 私有供应商接入
- 实验性或站点抓取型实现

这次只迁移了前一类里最适合先落 Rust 版本的部分，避免把不可复用、不可公开、维护成本高的实现包装成“统一 API”。

## 小鳄鱼文档

仓库已经补了 `xiaoeyu.config.json` 和 README 收录规则，后续可以直接用小鳄鱼把 root README 和 crate README 生成成站点文档。

文档接入说明见：

- [docs/README.md](https://github.com/zjarlin/addzero-lib-rust/blob/main/docs/README.md)

## 仓库文档范围

小鳄鱼当前会收录：

- 根目录 `README.md`
- `crates/**/README.md`

这次新增的 `addzero-creates` 音乐、Suno、天眼查用法，也会跟着这两个 README 一起被小鳄鱼站点收录。

默认不会收录：

- `docs/**`
- `target/**`
- 未来你明确标记为内部或实验用途的 README

# addzero-lib-rust

`addzero-lib-rust` 是一个以 Rust 为核心的工具型工作区，旨在把常用、可复用、低耦合的能力沉淀成独立的 crate 和轻量级应用。

本仓库聚焦于：

- `crates` 目录下的可复用基础库与领域能力
- `apps` 目录下的 Rust 应用与服务组合
- 通过文档、配置等辅助工作流的统一维护

## 核心目标

1. 构建可在多项目中复用的 Rust 库
2. 保持各模块边界清晰、依赖轻量
3. 支持网络、存储、算法、数据、UI 与运行时集成
4. 为 `az-aio` / `Codex` 等上层应用提供基础能力

## 目录结构概览

- `crates/algorithm/`：基于 ONNX、图像与视频的推理与检测能力
- `crates/api/`：对外 API 聚合、SDK 和协议兼容层
- `crates/core/`：基础通用类型、错误、配置、反射、上下文等
- `crates/data/`：数据模型、知识库、持久化、表格与区域数据处理
- `crates/network/`：HTTP、SMTP、MQTT、SSH、Drive/WebDAV 等网络协议与客户端
- `crates/storage/`：对象存储、MinIO、Drive 存储能力
- `crates/runtime/`：插件、脚本、桌面集成、启动器与运行时扩展
- `crates/text/`：文本处理、词典、OCR、语言与 i18n 工具
- `crates/ui/`：Dioxus 组件与 UI 体验基础
- `apps/`：实际运行应用、后台服务与插件宿主

## 重要子系统

- `az-creates`：HTTP API 创建器集合，已包含 Maven Central、网易云音乐、Suno、天眼查、Temp Mail 等
- `az-music`：音乐搜索、歌词、Suno 音乐生成等领域能力
- `az-curl`：curl 命令解析与请求构建辅助
- `az-email`：SMTP 邮件发送、附件与消息处理
- `az-rustfs`：Rust S3 兼容对象存储客户端
- `az-minio`：`az-rustfs` 上的 MinIO 便捷封装
- `az-mqtt`：MQTT 客户端与消息辅助
- `az-ssh`：SSH 远程命令执行与文件传输
- `az-excel`：Rust 原生 `.xlsx` 读写与结构化表格处理
- `az-ai-chat` / `az-aio-client`：AI 聊天与 AIO 客户端能力
- `az-drive-*`：Drive 存储、WebDAV 与同步能力

## 快速开始

克隆仓库后，可以直接在根目录运行：

```bash
cargo test
```

如果只想运行某个 crate 的测试，例如 `az-creates`：

```bash
cargo test -p az-creates
```

如果需要构建整个 workspace：

```bash
cargo build
```

## 示例：`az-creates` 用法

```rust
use az_creates::Creates;

fn main() -> anyhow::Result<()> {
    let api = Creates::maven_central()?;
    let latest = api.get_latest_version("com.google.guava", "guava")?;
    println!("latest guava version: {latest:?}");
    Ok(())
}
```

## 运行桌面/应用

仓库包含多个应用和插件宿主。当前主要应用入口位于 `apps/az-aio/desktop`，但该路径在 workspace 配置中排除，需根据实际项目需求单独运行。

如果你要运行其它应用，可查找对应应用目录下的 `Cargo.toml`。

## 文档与贡献

本仓库配合 `xiaoeyu.config.json`、`docs/` 目录和各 crate README 进行文档收录。

- 根 README：`README.md`
- 各 crate README：`crates/**/README.md`
- 各应用 README：`apps/**/README.md`

如果你需要补充文档或示例，优先在对应 crate/app 目录中添加 README，并保持内容可读、示例可复制。

## 代码风格与约定

- 采用 Rust 2024 Edition
- `cargo test` 作为首选验证方式
- 依赖轻量、模块边界清晰、避免跨领域耦合

## 许可

本仓库采用 `MIT OR Apache-2.0` 许可。

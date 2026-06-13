# az-drive-agent

无头实时网盘同步代理守护进程，通过轮询循环监听本地托管路径，自动与远程 Drive 进行双向对账与冲突处理。

## 功能

- 无头守护进程设计，不依赖 GUI
- 自动轮询本地托管路径并检测文件变更
- 双向同步：本地上传与远程拉取
- 冲突自动检测并写入冲突副本，无需手动 Git 式人工干预
- 支持多 owner Drive 融合与远程条目自动物化
- 基于 `.gitignore` 规则过滤无需同步的文件
- 支持 `.aioignore` 风格的跨设备取消托管黑名单
- 支持同步队列、失败重试和冲突挂起
- `state.json` 跨进程写锁，避免 Finder / daemon 并发覆盖本地状态
- 文件级冲突解决（保留远程 / 保留本地 / 使用合并文件）
- 设备级本地状态持久化

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-drive-agent = { path = "../az-drive-agent" }       # workspace 内部引用
# 或发布后：
# az-drive-agent = "0.1"                                # crates.io 引用
```

## 用法

```rust,ignore
use az_drive_agent::{
    agent::{DriveAgent, DriveAgentConfig},
    local_state::LocalStateStore,
};
use std::sync::Arc;

let config = DriveAgentConfig::new("user-demo", "device-id".into(), "我的笔记本".into());
let state_store = LocalStateStore::new(LocalStateStore::default_path());
// 需要提供 DriveMetadataStore 和 DriveObjectStore 的具体实现
let metadata = Arc::new(metadata_store);
let objects = Arc::new(object_store);
let agent = DriveAgent::new(metadata, objects, state_store, config);
```

如果需要在宿主侧接入 Git Pool 或其他显式同步协调器，使用 `DriveAgent::new_with_sync(...)`。

## 与 `aio drive` 的关系

这个 crate 是 `aio drive` 和 `az-drive-app` 的核心同步代理：

- `host_path()` / `unhost_path()`：托管与取消托管
- `list_tracked()`：`aio drive ls`
- `sync_once()`：单轮双向同步
- `sync_queue()` / `retry_sync_queue()`：耐久化同步队列
- `conflicts()` / `resolve_conflict()`：冲突查看与恢复

用户态命令与 Finder 集成都应该复用这层能力，而不是再造第二套同步逻辑。

## 依赖的 crates

- `az-drive-core` — Drive 核心路径、哈希与冲突决策逻辑
- `az-drive-store` — Drive 元数据存储与对象存储抽象
- `chrono` — 时间戳处理
- `dirs` — 平台配置目录解析
- `fs2` — 本地状态文件锁
- `ignore` — Gitignore 风格文件过滤
- `serde` / `serde_json` — 本地状态序列化
- `anyhow` — 错误返回与上下文
- `tokio` — 异步运行时
- `uuid` — 设备 ID 生成

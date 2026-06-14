# az-remote-session

基于 QUIC 协议的远程会话中继服务，负责设备注册、会话协商及远端数据同步。

## 功能

- 设备注册与在线状态管理
- 远程会话请求、授权与拒绝流程
- 剪贴板内容跨设备同步
- 视频帧实时推送
- 文件传输暂存
- 可配置的中继运行时参数（绑定地址、并发上限、空闲超时）

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-remote-session = { path = "../az-remote-session" }       # workspace 内部引用
# 或发布后：
# az-remote-session = "0.1"                                  # crates.io 引用
```

## 用法

```rust
use az_remote_session::api::{RemoteRelayService, RelayRuntimeConfig};

let mut relay = RemoteRelayService::new();

// 注册设备后即可发起会话请求、授权会话、推送剪贴板/视频帧等操作
let config = RelayRuntimeConfig::default();
```

## 依赖的 crates

- `az-remote-model` — 远程会话领域模型（设备描述、会话状态、剪贴板/帧/文件传输载荷）
- `az-remote-protocol` — 远程通信协议定义

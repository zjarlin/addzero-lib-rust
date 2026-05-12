# az-remote-host

远程控制宿主端的平台抽象层，负责检测当前操作系统、生成设备描述符并提供平台特定的权限提示。

---

## 功能

- **平台检测**：运行时自动识别 macOS、Windows、Linux（X11 / Wayland）和浏览器环境。
- **设备描述生成**：通过 `HostPlatformAdapter` trait 统一输出 `DeviceDescriptor`，包含设备 ID、平台类型、角色、在线状态与完整会话能力。
- **平台权限提示**：为每种操作系统提供对应的权限说明文字，方便 UI 层直接展示。
- **Wayland 感知**：在 Linux 下读取 `XDG_SESSION_TYPE` 区分 Wayland 与 X11，并自动附加兼容性备注。
- **Mock 实现**：内置 `MockHostPlatformAdapter`，可直接用于测试或快速原型开发。
- **统一错误类型**：`HostError` 与 `HostResult<T>` 提供一致的错误处理接口。

## 安装

在项目的 `Cargo.toml` 中添加依赖：

### 本地路径方式（monorepo 内部引用）

```toml
[dependencies]
az-remote-host = { path = "crates/runtime/az-remote-host" }
```

### crates.io 方式

```toml
[dependencies]
az-remote-host = "0.1"
```

## 用法

```rust
use az_remote_host::{current_platform, HostPlatformAdapter, MockHostPlatformAdapter};

fn main() {
    // 检测当前运行平台
    let platform = current_platform();
    println!("当前平台: {:?}", platform);

    // 使用 Mock 实现获取设备描述
    let adapter = MockHostPlatformAdapter;
    let descriptor = adapter.descriptor("我的主机").unwrap();
    println!("设备 ID: {}", descriptor.device_id);
    println!("设备名称: {}", descriptor.device_name);

    // 获取该平台对应的权限提示
    let hint = adapter.permission_hint();
    println!("权限提示: {}", hint);
}
```

## 依赖的 crates

| 依赖 | 说明 |
|------|------|
| `az-remote-model` | 共享数据模型层，提供 `DeviceDescriptor`、`DeviceRole`、`OnlineStatus`、`RemotePlatform`、`SessionCapability` 等类型 |
| `chrono` | 日期时间处理（启用 `serde` 特性），用于 `last_seen_at` 时间戳 |
| `thiserror` | 派生宏定义 `HostError` 枚举的 `Display` / `Error` 实现 |
| `uuid` | 生成设备唯一标识符（`Uuid::new_v4()`） |

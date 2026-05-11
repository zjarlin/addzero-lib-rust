# az-remote-protocol

远程桌面控制协议帧定义，基于 JSON 序列化，覆盖设备发现、会话协商、音视频传输、输入事件和剪贴板同步。

## 功能

- 统一的 `ControlFrame` 枚举，覆盖远程桌面全流程消息类型
- `StreamKind` 标识多路复用通道类型（Control / Video / Input / Clipboard / File）
- `to_json_bytes()` / `from_json_bytes()` 便捷的 JSON 二进制序列化/反序列化
- 敏感字段（relay_token）`Debug` 输出自动脱敏
- 全 crate 禁止 `unsafe`

## 安装

在 `Cargo.toml` 中添加：

\`\`\`toml
[dependencies]
az-remote-protocol = { path = "../az-remote-protocol" }   # workspace 内部引用
# 或发布后：
# az-remote-protocol = "0.1"                               # crates.io 引用
\`\`\`

## 用法

\`\`\`rust
use az_remote_protocol::{ControlFrame, DeviceHello};
use az_remote_model::DeviceDescriptor;

// 构建设备握手帧
let hello = DeviceHello {
    device: DeviceDescriptor {
        id: "device-001".into(),
        name: "我的电脑".into(),
        os: "Linux".into(),
        ..Default::default()
    },
    relay_token: "secret-token".into(),
};

let frame = ControlFrame::Hello(hello);

// 序列化为 JSON 字节
let bytes = frame.to_json_bytes().unwrap();

// 从 JSON 字节反序列化
let restored = ControlFrame::from_json_bytes(&bytes).unwrap();
assert_eq!(frame, restored);
\`\`\`

### 心跳帧

\`\`\`rust
use az_remote_protocol::ControlFrame;

let heartbeat = ControlFrame::Heartbeat;
let bytes = heartbeat.to_json_bytes().unwrap();
\`\`\`

## 依赖的 crates

- `az-remote-model` — 远程桌面共享数据模型（设备描述、会话请求/授权、输入事件、剪贴板、文件传输、视频帧）
- `serde` — 序列化/反序列化框架
- `serde_json` — JSON 格式编解码
- `thiserror` — 错误类型派生

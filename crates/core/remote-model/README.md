# az-remote-model

远程桌面控制的协议数据模型定义库，涵盖设备描述、会话管理、输入事件、视频帧、文件传输和剪贴板同步等全部数据类型。

## 功能

- `DeviceDescriptor`：设备信息描述（ID、名称、平台、角色、能力、在线状态）
- `SessionRequest` / `SessionGrant` / `SessionSummary`：会话建立与状态管理全流程
- `RemoteInputEvent`：远程输入事件（鼠标移动/点击/滚轮、键盘按键、文本输入）
- `VideoFrameEnvelope`：视频帧信封，支持 JPEG/PNG 编码，含序列号与时间戳
- `FileTransferEnvelope`：分块文件传输信封
- `ClipboardPayload`：剪贴板文本同步载荷
- `RemotePlatform`：支持 macOS、Windows、Linux (X11/Wayland)、Browser 五种平台
- `DeviceRole`：Host（被控端）与 Viewer（控制端）角色区分
- 所有类型均实现 `Serialize` / `Deserialize`，可直接用于 JSON 消息交换

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-remote-model = { path = "../remote-model" }   # workspace 内部引用
# 或发布后：
# az-remote-model = "0.1"                            # crates.io 引用
```

## 用法

```rust
use az_remote_model::contract::{
    DeviceDescriptor, DeviceRole, OnlineStatus, RemoteInputEvent, RemotePlatform,
    SessionCapability, SessionRequest,
};
use chrono::Utc;
use uuid::Uuid;

// 描述一台设备
let device = DeviceDescriptor {
    device_id: Uuid::new_v4(),
    device_name: "MacBook Pro".into(),
    platform: RemotePlatform::MacOs,
    role: DeviceRole::Host,
    capabilities: SessionCapability::full_host(),
    online_status: OnlineStatus::Online,
    last_seen_at: Utc::now(),
    notes: None,
};

// 构造会话请求
let request = SessionRequest {
    session_id: Uuid::new_v4(),
    viewer_id: Uuid::new_v4(),
    host_id: device.device_id,
    capability: SessionCapability::web_viewer(),
    requested_at: Utc::now(),
};

// 发送输入事件
let event = RemoteInputEvent::PointerMove { x: 100, y: 200 };
let json = serde_json::to_string(&event).unwrap();
```

## 依赖的 crates

- `chrono` — 日期时间类型（`DateTime<Utc>`）
- `serde` / `serde_json` — JSON 序列化与反序列化
- `uuid` — 设备/会话/传输的唯一标识符

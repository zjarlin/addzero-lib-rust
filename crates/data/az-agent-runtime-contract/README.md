# az-agent-runtime-contract

Agent 运行时与服务端之间的 API 契约类型定义，涵盖配对、节点管理、技能同步、冲突解决和会话认证。

## 功能

- **Agent 配对流程**：创建配对会话、轮询审批状态、交换节点凭证（`PairingRequest` / `PairingCreateResponse` / `PairingExchangeResponse`）
- **节点管理**：Agent 节点的注册、心跳上报、状态追踪（`AgentNode` / `AgentHeartbeat`）
- **制品分发**：Agent 安装包元数据管理，支持 macOS 二进制和 Docker Compose 两种渠道（`AgentArtifact` / `AgentArtifactChannel`）
- **技能同步**：客户端与服务端之间的技能快照比对、上传/下载、冲突检测（`SkillSnapshot` / `SkillSyncRequest` / `SkillSyncResponse`）
- **冲突解决**：技能版本冲突时的策略选择（以 Web 端或 Agent 端为准）（`SkillConflict` / `ConflictResolution`）
- **会话认证**：登录请求和会话用户信息（`LoginRequest` / `SessionUser`）
- **汇总视图**：聚合所有运行时状态的总览接口（`AgentRuntimeOverview`）

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-agent-runtime-contract = { path = "../az-agent-runtime-contract" }  # workspace 内部引用
# 或发布后：
# az-agent-runtime-contract = "0.1"                                      # crates.io 引用
```

## 用法

```rust
use az_agent_runtime_contract::*;
use uuid::Uuid;
use chrono::Utc;

// 创建配对请求
let request = PairingRequest {
    channel: AgentArtifactChannel::MacosBinary,
    device_name: "MacBook Pro".into(),
    platform: "macos".into(),
    agent_version: "1.0.0".into(),
};

// 技能同步请求
let sync_request = SkillSyncRequest {
    node_token: "token-xxx".into(),
    fs_root: "~/.hermes/skills".into(),
    skills: vec![SkillSnapshot {
        name: "example-skill".into(),
        keywords: vec!["example".into()],
        description: "示例技能".into(),
        body: "技能内容".into(),
        content_hash: "abc123".into(),
        updated_at: Some(Utc::now()),
    }],
};
```

## 依赖的 crates

- `serde` - 序列化/反序列化支持
- `chrono` - 日期时间类型（启用 `serde` feature）
- `uuid` - 通用唯一标识符（启用 `serde` 和 `v4` feature）

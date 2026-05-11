# az-kiro-auth-support

从 Python `KiroRegister` 项目提取的纯 Rust 安全支持代码，提供 Kiro/AWS Builder ID 设备授权流程的可审计构建块。

## 功能

- **设备授权流程管理** — `KiroDeviceFlowManager` 驱动客户端注册、设备授权请求、令牌轮询的完整生命周期
- **身份信息生成** — `generate_english_name` 生成随机英文姓名，`generate_password` 按策略生成密码
- **验证码解析** — `extract_verification_code` 从邮件文本中提取数字验证码
- **OIDC 配置** — `KiroOidcConfig` 封装 OpenID Connect 端点与客户端参数
- **纯安全 Rust** — `#[forbid(unsafe_code)]`，不移植浏览器指纹伪装等不可审计功能

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-kiro-auth-support = { path = "../az-kiro-auth-support" }       # workspace 内部引用
# 或发布后：
# az-kiro-auth-support = "0.1"                                     # crates.io 引用
```

## 用法

```rust
use az_kiro_auth_support::{KiroOidcConfig, KiroDeviceFlowManager};

// 构建 OIDC 配置
let config = KiroOidcConfig {
    // ... 填入实际 OIDC 端点参数
};

// 启动设备授权流程
// let mut manager = KiroDeviceFlowManager::new(config);
// let session = manager.start_device_flow().await?;
```

## 依赖的 crates

- `reqwest` — HTTP 客户端
- `ring` — 加密操作
- `serde` / `serde_json` — 序列化
- `regex` — 正则匹配（验证码提取）
- `uuid` — UUID 生成
- `thiserror` — 错误类型派生

#![forbid(unsafe_code)]

//! 从 Python `KiroRegister` 项目提取的纯 Rust 安全支持代码。
//!
//! 本 crate 提供可审计的基础构建块：Kiro/AWS Builder ID 设备授权流程请求、
//! 轮询状态管理、验证码解析以及本地测试数据生成。
//! 故意不移植 Camoufox 风格的浏览器指纹伪装或全自动第三方账号创建功能。
//!
//! ## 主要模块
//!
//! - `device_flow` — 设备授权流程管理，包含注册、轮询、令牌响应等核心类型
//! - `identity` — 英文姓名生成与密码策略工具
//! - `otp` — 邮件验证码提取
//! - `config` — OIDC 客户端配置
//!
//! ## 关键特性
//!
//! - `KiroDeviceFlowManager` 驱动设备授权流程的完整生命周期
//! - `generate_english_name` / `generate_password` 提供身份信息生成能力
//! - `extract_verification_code` 从邮件文本中解析验证码
//! - `#[forbid(unsafe_code)]` 保证全 crate 无 unsafe 代码

mod config;
mod device_flow;
mod error;
mod http;
mod identity;
mod otp;
mod unsupported;

pub use config::{KiroOidcConfig, KiroOidcConfigBuilder};
pub use device_flow::{
    KiroClientRegistration, KiroDeviceAuthorization, KiroDeviceFlow, KiroDeviceFlowClient,
    KiroDeviceFlowManager, KiroDeviceFlowSession, KiroDeviceFlowSessionSnapshot,
    KiroDeviceFlowSessionStatus, KiroLoginType, KiroTokenPoll, KiroTokenResponse,
};
pub use error::{KiroAuthSupportError, KiroAuthSupportResult};
pub use identity::{
    EnglishName, EnglishNameOptions, NameGender, PasswordPolicy, generate_english_name,
    generate_password,
};
pub use otp::extract_verification_code;
pub use unsupported::{BlockedCapability, unsupported_capability};

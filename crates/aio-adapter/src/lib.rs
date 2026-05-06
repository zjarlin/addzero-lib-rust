//! AIO 适配层 — 双端桥接。
//!
//! 提供：
//! - Web API → 引擎/插件 调用桥接
//! - Tauri 桌面 ←→ 内核 通信代理
//! - 统一数据协议序列化/反序列化
//! - 抹平 Web/Desktop 差异

/// Placeholder — will contain:
/// - Web API handlers that delegate to aio-engine and aio-runtime
/// - Tauri command implementations
/// - Shared protocol types (WebSocket messages, API DTOs)
pub mod protocol {
    //! Shared protocol types for web ↔ engine communication.
}

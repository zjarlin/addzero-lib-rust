//! 基于类型的日志目标生成工具。
//!
//! 为 [`log`] crate 提供按 Rust 类型自动生成日志目标（target）的能力。
//! 通过缓存 `TypeId → type_name` 映射，避免重复的类型名称解析开销。
//!
//! # 核心功能
//!
//! - [`logger_target::<T>()`] — 为泛型类型 `T` 生成并缓存日志目标字符串
//! - [`value_logger_target()`] — 为任意值动态获取类型名称作为日志目标
//! - 便捷日志宏：[`trace_for!`]、[`debug_for!`]、[`info_for!`]、[`warn_for!`]、[`error_for!`]
//!   以目标类型/值为中心发起日志调用，等价于 `log::trace!(target: ..., ...)`
//!
//! # 使用场景
//!
//! 当希望日志输出的 target 字段自动对应模块路径或类型全名时，
//! 使用本 crate 可以避免手动拼写字符串，同时保持类型与日志目标的一致性。

use std::any::{TypeId, type_name, type_name_of_val};
use std::collections::HashMap;
use std::sync::{OnceLock, RwLock};

fn logger_map() -> &'static RwLock<HashMap<TypeId, &'static str>> {
    static LOGGER_MAP: OnceLock<RwLock<HashMap<TypeId, &'static str>>> = OnceLock::new();
    LOGGER_MAP.get_or_init(|| RwLock::new(HashMap::new()))
}

pub fn logger_target<T>() -> &'static str
where
    T: 'static,
{
    let type_id = TypeId::of::<T>();
    if let Some(target) = {
        let read_guard = match logger_map().read() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };
        read_guard.get(&type_id).copied()
    } {
        return target;
    }

    let target = type_name::<T>();
    let mut write_guard = match logger_map().write() {
        Ok(guard) => guard,
        Err(poisoned) => poisoned.into_inner(),
    };
    write_guard.insert(type_id, target);
    target
}

pub fn value_logger_target<T>(value: &T) -> &'static str
where
    T: ?Sized,
{
    type_name_of_val(value)
}

#[macro_export]
macro_rules! trace_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::trace!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! debug_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::debug!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! info_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::info!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! warn_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::warn!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

#[macro_export]
macro_rules! error_for {
    ($value:expr, $($arg:tt)+) => {
        ::log::error!(target: $crate::value_logger_target(&$value), $($arg)+)
    };
}

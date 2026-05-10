//! Rust 运行时反射与缓存工具库。
//!
//! 提供类型元信息提取（字段名、描述、列名、嵌套结构遍历）、带 TTL 过期的线程安全缓存，
//! 以及 JSON 值层面的便捷判断辅助函数。主要用于 ORM 映射、动态表单生成、
//! AI 结构化输出等需要在运行时感知类型结构的场景。
//!
//! ## 主要模块
//!
//! - [`metainfo`] — [`MetaInfo`] trait 及 [`FieldInfo`] 描述结构，支持嵌套对象与集合的递归展开；
//!   另含 `extract_table_name`（从 SQL 中提取表名）与 `guess_column_name`（驼峰 → 蛇形）。
//! - [`cache`] — [`ExpiringCache`]：基于 `Mutex` 的线程安全 KV 缓存，支持单条 TTL 与最大容量淘汰。
//! - [`value`] — JSON 值判断辅助：`is_new` / `is_not_new`（检测"空白"对象）、集合与自定义对象类型判断。
//!
//! ## 宏
//!
//! - [`field_info!`] — 声明式构造 [`FieldInfo`]，支持叶子字段、嵌套对象、集合以及列名/描述注解。
//! - [`reflect_meta!`] — 为任意类型实现 [`MetaInfo`] trait，可选附带类型描述。

mod cache;
mod metainfo;
mod value;

pub use cache::{CacheError, ExpiringCache};
pub use metainfo::{
    FieldInfo, FieldInfoSimple, MetaInfo, extract_table_name, get_field_infos,
    get_simple_field_info_str, guess_column_name,
};
pub use value::{
    contains_ignore_order, is_collection_value, is_custom_object_value, is_new, is_not_new,
};

#[macro_export]
macro_rules! field_info {
    ($name:ident : $ty:ty) => {
        $crate::FieldInfo::leaf(stringify!($name), None, None, stringify!($ty))
    };
    ($name:ident : $ty:ty, column = $column:expr) => {
        $crate::FieldInfo::leaf(stringify!($name), None, Some($column), stringify!($ty))
    };
    ($name:ident : $ty:ty => $description:expr) => {
        $crate::FieldInfo::leaf(stringify!($name), Some($description), None, stringify!($ty))
    };
    ($name:ident : $ty:ty => $description:expr, column = $column:expr) => {
        $crate::FieldInfo::leaf(
            stringify!($name),
            Some($description),
            Some($column),
            stringify!($ty),
        )
    };
    ($name:ident : $ty:ty, nested = $nested:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            None,
            None,
            stringify!($ty),
            <$nested as $crate::MetaInfo>::field_infos(),
        )
    };
    ($name:ident : $ty:ty, column = $column:expr, nested = $nested:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            None,
            Some($column),
            stringify!($ty),
            <$nested as $crate::MetaInfo>::field_infos(),
        )
    };
    ($name:ident : $ty:ty => $description:expr, nested = $nested:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            Some($description),
            None,
            stringify!($ty),
            <$nested as $crate::MetaInfo>::field_infos(),
        )
    };
    ($name:ident : $ty:ty => $description:expr, column = $column:expr, nested = $nested:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            Some($description),
            Some($column),
            stringify!($ty),
            <$nested as $crate::MetaInfo>::field_infos(),
        )
    };
    ($name:ident : $ty:ty, collection = $item:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            None,
            None,
            stringify!($ty),
            <$item as $crate::MetaInfo>::field_infos(),
        )
    };
    ($name:ident : $ty:ty, column = $column:expr, collection = $item:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            None,
            Some($column),
            stringify!($ty),
            <$item as $crate::MetaInfo>::field_infos(),
        )
    };
    ($name:ident : $ty:ty => $description:expr, collection = $item:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            Some($description),
            None,
            stringify!($ty),
            <$item as $crate::MetaInfo>::field_infos(),
        )
    };
    ($name:ident : $ty:ty => $description:expr, column = $column:expr, collection = $item:ty) => {
        $crate::FieldInfo::nested(
            stringify!($name),
            Some($description),
            Some($column),
            stringify!($ty),
            <$item as $crate::MetaInfo>::field_infos(),
        )
    };
}

#[macro_export]
macro_rules! reflect_meta {
    ($ty:ty, description = $description:expr, [$($field:expr),* $(,)?]) => {
        impl $crate::MetaInfo for $ty {
            fn type_description() -> Option<&'static str> {
                Some($description)
            }

            fn field_infos() -> Vec<$crate::FieldInfo> {
                vec![$($field),*]
            }
        }
    };
    ($ty:ty, [$($field:expr),* $(,)?]) => {
        impl $crate::MetaInfo for $ty {
            fn field_infos() -> Vec<$crate::FieldInfo> {
                vec![$($field),*]
            }
        }
    };
}

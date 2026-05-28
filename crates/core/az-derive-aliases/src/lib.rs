#![doc = include_str!("../README.md")]

pub use macro_rules_attribute::apply;

#[doc(hidden)]
#[macro_export]
macro_rules! __az_derive_aliases_derive {
    (($($derive:path),+), $item:item) => {
        #[derive($($derive),+)]
        $item
    };
}

/// 带 `Debug` 和 `Deserialize` 的可反序列化响应/输入类型。
#[macro_export]
macro_rules! deserialize_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::serde::Deserialize), $item);
    };
}

/// 带 `Clone`、`Debug` 和 `Deserialize` 的可反序列化响应/输入类型。
#[macro_export]
macro_rules! deserialize_clone_debug {
    ($item:item) => {
        $crate::deserialize_debug! {
            #[derive(Clone)]
            $item
        }
    };
}

/// 带 camelCase serde 约定、`Clone`、`Debug` 和 `Deserialize` 的响应/输入类型。
#[macro_export]
macro_rules! deserialize_camel_clone_debug {
    ($item:item) => {
        $crate::deserialize_clone_debug! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 `Debug` 和 `Serialize` 的可序列化请求/输出类型。
#[macro_export]
macro_rules! serialize_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::serde::Serialize), $item);
    };
}

/// 带 `Clone`、`Debug` 和 `Serialize` 的可序列化请求/输出类型。
#[macro_export]
macro_rules! serialize_clone_debug {
    ($item:item) => {
        $crate::serialize_debug! {
            #[derive(Clone)]
            $item
        }
    };
}

/// 带 camelCase serde 约定、`Clone`、`Debug` 和 `Serialize` 的请求/输出类型。
#[macro_export]
macro_rules! serialize_camel_clone_debug {
    ($item:item) => {
        $crate::serialize_clone_debug! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带序列化、相等和调试 trait 的请求/输出类型。
#[macro_export]
macro_rules! serialize_eq {
    ($item:item) => {
        $crate::serialize_clone_debug! {
            #[derive(PartialEq, Eq)]
            $item
        }
    };
}

/// 带 `Copy`、序列化、相等和调试 trait 的小型请求/输出类型。
#[macro_export]
macro_rules! serialize_copy_eq {
    ($item:item) => {
        $crate::serialize_eq! {
            #[derive(Copy)]
            $item
        }
    };
}

/// 带序列化、部分相等和调试 trait 的请求/输出类型。
#[macro_export]
macro_rules! serialize_partial_eq {
    ($item:item) => {
        $crate::serialize_clone_debug! {
            #[derive(PartialEq)]
            $item
        }
    };
}

/// 带 camelCase serde 约定、相等和调试 trait 的请求/输出类型。
#[macro_export]
macro_rules! serialize_camel_eq {
    ($item:item) => {
        $crate::serialize_eq! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 camelCase serde 约定、部分相等和调试 trait 的请求/输出类型。
#[macro_export]
macro_rules! serialize_camel_partial_eq {
    ($item:item) => {
        $crate::serialize_partial_eq! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 kebab-case serde 约定、相等和调试 trait 的数据类型。
#[macro_export]
macro_rules! serde_kebab_eq {
    ($item:item) => {
        $crate::serde_eq! {
            #[serde(rename_all = "kebab-case")]
            $item
        }
    };
}

/// 带 UPPERCASE serde 约定、相等和调试 trait 的数据类型。
#[macro_export]
macro_rules! serde_upper_eq {
    ($item:item) => {
        $crate::serde_eq! {
            #[serde(rename_all = "UPPERCASE")]
            $item
        }
    };
}

/// 带反序列化、相等和调试 trait 的响应/输入类型。
#[macro_export]
macro_rules! deserialize_eq {
    ($item:item) => {
        $crate::deserialize_clone_debug! {
            #[derive(PartialEq, Eq)]
            $item
        }
    };
}

/// 带反序列化、部分相等和调试 trait 的响应/输入类型。
#[macro_export]
macro_rules! deserialize_partial_eq {
    ($item:item) => {
        $crate::deserialize_clone_debug! {
            #[derive(PartialEq)]
            $item
        }
    };
}

/// 带 camelCase serde 约定、反序列化、相等和调试 trait 的响应/输入类型。
#[macro_export]
macro_rules! deserialize_camel_eq {
    ($item:item) => {
        $crate::deserialize_eq! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 camelCase serde 约定、反序列化、部分相等和调试 trait 的响应/输入类型。
#[macro_export]
macro_rules! deserialize_camel_partial_eq {
    ($item:item) => {
        $crate::deserialize_partial_eq! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 `derive_more::From`、调试和相等 trait 的轻量转换类型。
#[macro_export]
macro_rules! from_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, ::derive_more::From, PartialEq), $item);
    };
}

/// 带 `derive_more::From` 和完整相等 trait 的轻量转换类型。
#[macro_export]
macro_rules! from_plain_eq {
    ($item:item) => {
        $crate::from_eq! {
            #[derive(Eq)]
            $item
        }
    };
}

/// 带 `Copy`、`derive_more::From` 和完整相等 trait 的轻量转换类型。
#[macro_export]
macro_rules! from_copy_eq {
    ($item:item) => {
        $crate::from_plain_eq! {
            #[derive(Copy)]
            $item
        }
    };
}

/// 带 `derive_more::From` 和 `Display` 的轻量转换/值类型。
#[macro_export]
macro_rules! from_display {
    ($item:item) => {
        $crate::from_eq! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 `thiserror` 以及常用相等/调试 trait 的可比较错误类型。
#[macro_export]
macro_rules! error_eq {
    ($item:item) => {
        $crate::error! {
            #[derive(Clone, PartialEq, Eq)]
            $item
        }
    };
}

/// 带 `thiserror`、`Copy` 以及常用相等/调试 trait 的小型可比较错误类型。
#[macro_export]
macro_rules! error_copy_eq {
    ($item:item) => {
        $crate::error_eq! {
            #[derive(Copy)]
            $item
        }
    };
}

/// 带 `thiserror` 和调试格式化的基础库错误类型。
#[macro_export]
macro_rules! error {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::thiserror::Error), $item);
    };
}

/// 通过显式源 pattern 到目标表达式的映射实现 `From`。
#[macro_export]
macro_rules! impl_from_match {
    ($source:ty => $target:ty { $($pattern:pat => $expr:expr),+ $(,)? }) => {
        impl From<$source> for $target {
            fn from(value: $source) -> Self {
                match value {
                    $($pattern => $expr,)+
                }
            }
        }
    };
}

/// 实现一个 const 固有方法，将配置枚举 pattern 映射到 kind 表达式。
#[macro_export]
macro_rules! impl_enum_kind {
    ($source:ty => $target:ty, $method:ident { $($pattern:pat => $kind:expr),+ $(,)? }) => {
        impl $source {
            #[must_use]
            pub const fn $method(&self) -> $target {
                match self {
                    $($pattern => $kind,)+
                }
            }
        }
    };
}

/// 通过返回显式表达式实现 `Default`。
#[macro_export]
macro_rules! impl_default {
    ($target:ty => $expr:expr $(,)?) => {
        impl Default for $target {
            fn default() -> Self {
                $expr
            }
        }
    };
}

/// 为“一个字段来自源值，其余字段使用 `Default`”的结构体实现 `From`。
#[macro_export]
macro_rules! impl_from_with_default {
    ($source:ty => $target:ty { $field:ident: |$value:ident| $expr:expr $(,)? }) => {
        impl From<$source> for $target {
            fn from($value: $source) -> Self {
                Self {
                    $field: $expr,
                    ..Default::default()
                }
            }
        }
    };
}

/// 通过委托到 `Self::parse` 等固有解析函数实现 `FromStr`。
#[macro_export]
macro_rules! impl_from_str_parse {
    ($target:ty => $error:ty, $parse:path $(,)?) => {
        impl ::std::str::FromStr for $target {
            type Err = $error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                $parse(value)
            }
        }
    };
    ($target:ty => $error:ty $(,)?) => {
        impl ::std::str::FromStr for $target {
            type Err = $error;

            fn from_str(value: &str) -> Result<Self, Self::Err> {
                <$target>::parse(value)
            }
        }
    };
}

/// 通过委托到 `Self::parse` 等固有解析函数实现 `TryFrom<&str>`。
#[macro_export]
macro_rules! impl_try_from_str_parse {
    ($target:ty => $error:ty, $parse:path $(,)?) => {
        impl ::core::convert::TryFrom<&str> for $target {
            type Error = $error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                $parse(value)
            }
        }
    };
    ($target:ty => $error:ty $(,)?) => {
        impl ::core::convert::TryFrom<&str> for $target {
            type Error = $error;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                <$target>::parse(value)
            }
        }
    };
}

/// 带 serde、相等和调试 trait 的数据类型。
#[macro_export]
macro_rules! serde_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                PartialEq,
                Eq,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// 带 camelCase serde 约定、相等和调试 trait 的数据类型。
#[macro_export]
macro_rules! serde_camel_eq {
    ($item:item) => {
        $crate::serde_eq! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 serde、相等、调试和 hash trait 的标识类型。
#[macro_export]
macro_rules! serde_eq_hash {
    ($item:item) => {
        $crate::serde_eq! {
            #[derive(Hash)]
            $item
        }
    };
}

/// 带 serde、相等、调试、hash 和 `Display` 的标识类型。
#[macro_export]
macro_rules! serde_eq_hash_display {
    ($item:item) => {
        $crate::serde_eq_hash! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 serde、相等、调试、hash 和排序 trait 的有序标识类型。
#[macro_export]
macro_rules! serde_eq_hash_ord {
    ($item:item) => {
        $crate::serde_eq_hash! {
            #[derive(PartialOrd, Ord)]
            $item
        }
    };
}

/// 带 serde、相等、调试、hash、排序和 `Display` 的有序标识类型。
#[macro_export]
macro_rules! serde_eq_hash_ord_display {
    ($item:item) => {
        $crate::serde_eq_hash_ord! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 serde、相等、hash、`Display` 和转发式 `AsRef` 的有序标识类型。
#[macro_export]
macro_rules! serde_eq_hash_ord_display_as_ref {
    ($item:item) => {
        $crate::serde_eq_hash_ord_display! {
            #[derive(::derive_more::AsRef)]
            #[as_ref(forward)]
            $item
        }
    };
}

/// 带 serde、排序、`Display`、`AsRef<str>` 和字符串 helper 的有序字符串值对象。
#[macro_export]
macro_rules! serde_string_value_object {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident(String);
    ) => {
        $crate::serde_eq_hash_ord_display_as_ref! {
            $(#[$meta])*
            $vis struct $name(String);
        }

        $crate::__az_derive_aliases_string_value_object_impl!($name);
    };
}

/// 不带 serde、带排序、`Display`、`AsRef<str>` 和字符串 helper 的字符串值对象。
#[macro_export]
macro_rules! plain_string_value_object {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident(String);
    ) => {
        $crate::plain_eq_hash_ord_display! {
            #[derive(::derive_more::AsRef)]
            #[as_ref(forward)]
            $(#[$meta])*
            $vis struct $name(String);
        }

        $crate::__az_derive_aliases_string_value_object_impl!($name);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __az_derive_aliases_string_value_object_impl {
    ($name:ident) => {
        impl $name {
            #[must_use]
            pub fn as_str(&self) -> &str {
                self.as_ref()
            }

            #[must_use]
            pub fn into_string(self) -> String {
                self.0
            }
        }
    };
}

/// 带 SQLx `FromRow` 的行映射类型。
#[macro_export]
macro_rules! sqlx_from_row {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((::sqlx::FromRow), $item);
    };
}

/// 带 Shaku `Component` 的组件类型。
#[macro_export]
macro_rules! shaku_component {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Component), $item);
    };
}

/// 带 serde 和相等 trait、但不生成 `Debug` 的数据类型。
#[macro_export]
macro_rules! serde_eq_no_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                PartialEq,
                Eq,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// 带 serde、相等 trait 和脱敏 `Debug` 的数据类型。
#[macro_export]
macro_rules! serde_eq_redacted {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                ::derive_more::Debug,
                PartialEq,
                Eq,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// 带 `Clone`、`Debug`、`Eq` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, Eq, PartialEq), $item);
    };
}

/// 仅带 `Clone` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_clone {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone), $item);
    };
}

/// 带 `Clone` 和 `Copy` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_copy {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Copy), $item);
    };
}

/// 仅带 `Default` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Default), $item);
    };
}

/// 仅带 `Debug` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug), $item);
    };
}

/// 带 `Clone` 和 `Debug` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_clone_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug), $item);
    };
}

/// 带 `Debug` 和 `Default` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default_debug {
    ($item:item) => {
        $crate::plain_debug! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 `Clone` 和 `Default` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default_clone {
    ($item:item) => {
        $crate::plain_clone! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug` 和 `Default` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default_clone_debug {
    ($item:item) => {
        $crate::plain_clone_debug! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_clone_debug_display {
    ($item:item) => {
        $crate::plain_clone_debug! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 `Clone`、`Eq` 和 `PartialEq`、但不生成 `Debug` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_eq_no_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Eq, PartialEq), $item);
    };
}

/// 带相等 trait 和脱敏 `Debug` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_eq_redacted {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, ::derive_more::Debug, Eq, PartialEq), $item);
    };
}

/// 带 `Clone` 和脱敏 `Debug` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_clone_redacted {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, ::derive_more::Debug), $item);
    };
}

/// 带 `Clone`、`Debug`、`Eq`、`Hash` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_eq_hash {
    ($item:item) => {
        $crate::plain_eq! {
            #[derive(Hash)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug`、`Eq`、`Hash`、`PartialEq` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_eq_hash_display {
    ($item:item) => {
        $crate::plain_eq_hash! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_partial_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, PartialEq), $item);
    };
}

/// 带 `Clone`、`Debug`、`PartialEq` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_partial_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, PartialEq, ::derive_more::Display),
            $item
        );
    };
}

/// 带 `Clone`、`Debug`、`Default`、`Eq` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default_eq {
    ($item:item) => {
        $crate::plain_eq! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug`、`Default` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default_partial_eq {
    ($item:item) => {
        $crate::plain_partial_eq! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 `Clone`、`Copy`、`Debug`、`Eq` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_copy_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Copy, Debug, Eq, PartialEq), $item);
    };
}

/// 带 `Clone`、`Copy`、`Debug`、`Eq`、`PartialEq` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_copy_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Copy, Debug, Eq, PartialEq, ::derive_more::Display),
            $item
        );
    };
}

/// 带 `Clone`、`Copy`、`Debug`、`Eq`、`Hash` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_copy_eq_hash {
    ($item:item) => {
        $crate::plain_copy_eq! {
            #[derive(Hash)]
            $item
        }
    };
}

/// 带 `Clone`、`Copy`、`Debug`、`Eq`、`Hash`、`PartialEq`、`PartialOrd`、`Ord` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_copy_eq_hash_ord_display {
    ($item:item) => {
        $crate::plain_copy_eq_hash_display! {
            #[derive(PartialOrd, Ord)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug`、`Eq`、`Hash`、`PartialEq`、`PartialOrd`、`Ord` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_eq_hash_ord_display {
    ($item:item) => {
        $crate::plain_eq_hash_display! {
            #[derive(PartialOrd, Ord)]
            $item
        }
    };
}

/// 带 `Clone`、`Copy`、`Debug`、`Eq`、`Hash`、`PartialEq` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_copy_eq_hash_display {
    ($item:item) => {
        $crate::plain_copy_eq_hash! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug`、`Eq`、`PartialEq` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, Eq, PartialEq, ::derive_more::Display),
            $item
        );
    };
}

/// 带 `Clone`、`Copy`、`Debug`、`Default`、`Eq` 和 `PartialEq` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default_copy_eq {
    ($item:item) => {
        $crate::plain_copy_eq! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 `Clone`、`Copy`、`Debug`、`Default`、`Eq`、`PartialEq` 和 `Display` 的纯内存结构类型。
#[macro_export]
macro_rules! plain_default_copy_eq_display {
    ($item:item) => {
        $crate::plain_default_copy_eq! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 `Debug` 和 Clap `Parser` 的解析器类型。
#[macro_export]
macro_rules! clap_parser {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::clap::Parser), $item);
    };
}

/// 带 `Debug` 和 Clap `Args` 的参数类型。
#[macro_export]
macro_rules! clap_args {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::clap::Args), $item);
    };
}

/// 带 `Debug`、Clap `Args`、`Clone` 和 `Default` 的参数类型。
#[macro_export]
macro_rules! clap_args_default_clone {
    ($item:item) => {
        $crate::clap_args! {
            #[derive(Clone, Default)]
            $item
        }
    };
}

/// 带 `Debug` 和 Clap `Subcommand` 的子命令类型。
#[macro_export]
macro_rules! clap_subcommand {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::clap::Subcommand), $item);
    };
}

/// 带 Dioxus `Props`、`Clone` 和 `PartialEq` 的属性类型。
#[macro_export]
macro_rules! dioxus_props {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((::dioxus::prelude::Props, Clone, PartialEq), $item);
    };
}

/// 带 `Debug`、`Clone`、`Copy` 和相等 trait 的 Clap value enum。
#[macro_export]
macro_rules! clap_value_enum {
    ($item:item) => {
        $crate::plain_copy_eq! {
            #[derive(::clap::ValueEnum)]
            $item
        }
    };
}

/// 带 `Clone`、`Debug`、`PartialEq` 和 SeaORM `DeriveEntityModel` 的实体模型。
#[macro_export]
macro_rules! seaorm_entity_model {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, PartialEq, DeriveEntityModel), $item);
    };
}

/// 带 `Clone`、`Debug`、`PartialEq`、`Eq` 和 SeaORM `DeriveEntityModel` 的实体模型。
#[macro_export]
macro_rules! seaorm_entity_model_eq {
    ($item:item) => {
        $crate::seaorm_entity_model! {
            #[derive(Eq)]
            $item
        }
    };
}

/// 带 SeaORM 关系派生、标准迭代和调试 trait 的关系枚举。
#[macro_export]
macro_rules! seaorm_relation {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Copy, Clone, Debug, EnumIter, DeriveRelation), $item);
    };
}

/// 带 serde、相等、调试和 `Copy` 的小型数据类型。
#[macro_export]
macro_rules! serde_eq_copy {
    ($item:item) => {
        $crate::serde_eq! {
            #[derive(Copy)]
            $item
        }
    };
}

/// 带 serde、相等、调试、`Copy` 和 `Display` 的小型数据类型。
#[macro_export]
macro_rules! serde_eq_copy_display {
    ($item:item) => {
        $crate::serde_eq_copy! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 `Copy`、`derive_more::From`、调试、相等和 `Display` 的值包装类型。
#[macro_export]
macro_rules! from_copy_eq_display {
    ($item:item) => {
        $crate::from_copy_eq! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 serde、相等、调试、`Copy`、`Default` 和 hash trait 的小型数据类型。
#[macro_export]
macro_rules! serde_eq_default_copy {
    ($item:item) => {
        $crate::serde_eq_copy! {
            #[derive(Default, Hash)]
            $item
        }
    };
}

/// 带 serde、相等、调试、`Copy`、`Default`、hash 和排序 trait 的有序小型数据类型。
#[macro_export]
macro_rules! serde_eq_default_copy_ord {
    ($item:item) => {
        $crate::serde_eq_default_copy! {
            #[derive(PartialOrd, Ord)]
            $item
        }
    };
}

/// 带 serde、相等、调试和 `Default` 的数据类型。
#[macro_export]
macro_rules! serde_eq_default {
    ($item:item) => {
        $crate::serde_eq! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 camelCase serde 约定、相等、调试和 `Default` 的数据类型。
#[macro_export]
macro_rules! serde_camel_eq_default {
    ($item:item) => {
        $crate::serde_eq_default! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 serde、部分相等和调试 trait 的数据类型。
#[macro_export]
macro_rules! serde_partial_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                PartialEq,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// 带 camelCase serde 约定、部分相等和调试 trait 的数据类型。
#[macro_export]
macro_rules! serde_camel_partial_eq {
    ($item:item) => {
        $crate::serde_partial_eq! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 snake_case serde 约定、部分相等和调试 trait 的数据类型。
#[macro_export]
macro_rules! serde_code_partial_eq {
    ($item:item) => {
        $crate::serde_partial_eq! {
            #[serde(rename_all = "snake_case")]
            $item
        }
    };
}

/// 带 serde、部分相等、调试和 `Default` 的数据类型。
#[macro_export]
macro_rules! serde_partial_eq_default {
    ($item:item) => {
        $crate::serde_partial_eq! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 camelCase serde 约定、部分相等、调试和 `Default` 的数据类型。
#[macro_export]
macro_rules! serde_camel_partial_eq_default {
    ($item:item) => {
        $crate::serde_partial_eq_default! {
            #[serde(rename_all = "camelCase")]
            $item
        }
    };
}

/// 带 serde、部分相等、调试和 `Display` 的数据类型。
#[macro_export]
macro_rules! serde_partial_eq_display {
    ($item:item) => {
        $crate::serde_partial_eq! {
            #[derive(::derive_more::Display)]
            $item
        }
    };
}

/// 带 snake_case serde、字符串转换和 hash 派生的代码类型。
#[macro_export]
macro_rules! serde_code {
    ($item:item) => {
        $crate::__az_derive_aliases_serde_code!("snake_case", $item);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __az_derive_aliases_serde_code {
    ($case:literal, $item:item) => {
        $crate::__az_derive_aliases_serde_code_with!((), ::strum::Display, $case, $item);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __az_derive_aliases_serde_code_with {
    (($($extra:path),* $(,)?), $display:path, $case:literal, $item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                $($extra,)*
                ::serde::Serialize,
                ::serde::Deserialize,
                $display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = $case)]
            #[strum(serialize_all = $case)]
            $item
        );
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __az_derive_aliases_code_enum_impl {
    ($name:ident) => {
        impl $name {
            pub const ALL: &'static [Self] = <Self as ::strum::VariantArray>::VARIANTS;

            #[must_use]
            pub fn as_str(self) -> &'static str {
                self.into()
            }

            #[must_use]
            pub fn code(self) -> &'static str {
                self.as_str()
            }

            pub fn from_code(value: &str) -> Option<Self> {
                value.parse().ok()
            }
        }
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __az_derive_aliases_code_default_enum_impl {
    ($name:ident) => {
        impl $name {
            #[must_use]
            pub fn from_code_or_default(value: &str) -> Self {
                Self::from_code(value).unwrap_or_default()
            }
        }
    };
}

/// 带 kebab-case serde、字符串转换和 hash 派生的代码类型。
#[macro_export]
macro_rules! serde_kebab_code {
    ($item:item) => {
        $crate::__az_derive_aliases_serde_code!("kebab-case", $item);
    };
}

/// 带 lowercase serde、字符串转换和 hash 派生的代码类型。
#[macro_export]
macro_rules! serde_lower_code {
    ($item:item) => {
        $crate::__az_derive_aliases_serde_code!("lowercase", $item);
    };
}

/// 带 snake_case serde、自定义 `Display` 和 hash 派生的代码类型。
#[macro_export]
macro_rules! serde_code_display {
    ($item:item) => {
        $crate::__az_derive_aliases_serde_code_with!(
            (),
            ::derive_more::Display,
            "snake_case",
            $item
        );
    };
}

/// 带 lowercase serde、字符串转换、变体列表和 `code` helper 的代码枚举。
#[macro_export]
macro_rules! serde_lower_code_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_lower_code! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带 snake_case serde、字符串转换、变体列表和 `code` helper 的代码枚举。
#[macro_export]
macro_rules! serde_code_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带 snake_case serde、`strum::EnumProperty`、变体列表和 `code` helper 的代码枚举。
#[macro_export]
macro_rules! serde_code_props_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_enum! {
            #[derive(::strum::EnumProperty)]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }
    };
}

/// 带 snake_case serde、自定义 `Display`、`strum::EnumProperty` 和 `code` helper 的代码枚举。
#[macro_export]
macro_rules! serde_code_display_props_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_display_enum! {
            #[derive(::strum::EnumProperty)]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }
    };
}

/// 带 kebab-case serde、字符串转换、变体列表和 `code` helper 的代码枚举。
#[macro_export]
macro_rules! serde_kebab_code_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_kebab_code! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带 snake_case serde、自定义 `Display`、变体列表和 `code` helper 的代码枚举。
#[macro_export]
macro_rules! serde_code_display_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_display! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带 snake_case 字符串转换、自定义 `Display` 和变体列表的纯内存代码枚举。
#[macro_export]
macro_rules! plain_code_display_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_derive_aliases_plain_code! {
            (Default, ::derive_more::Display),
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
        $crate::__az_derive_aliases_code_default_enum_impl!($name);
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! __az_derive_aliases_plain_code {
    (($($extra:path),* $(,)?), $item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Eq,
                PartialEq,
                ::core::hash::Hash,
                $($extra,)*
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[strum(serialize_all = "snake_case")]
            $item
        );
    };
}

/// 带 snake_case 字符串转换和变体列表的纯内存代码枚举。
#[macro_export]
macro_rules! plain_code_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_derive_aliases_plain_code! {
            (),
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带 `Default`、snake_case 字符串转换和变体列表的纯内存代码枚举。
#[macro_export]
macro_rules! plain_code_default_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::plain_code_enum! {
            #[derive(Default)]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_default_enum_impl!($name);
    };
}

/// 带自定义 `Display`、显式字符串转换和变体列表的纯内存代码枚举。
#[macro_export]
macro_rules! plain_code_display_no_default_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_derive_aliases_plain_code! {
            (::derive_more::Display),
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带自定义 `Display`、变体说明文案且不生成 `Default` 的纯内存代码枚举。
#[macro_export]
macro_rules! plain_code_display_message_no_default_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::plain_code_display_no_default_enum! {
            #[derive(::strum::EnumMessage)]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }
    };
}

/// 同时可作为 Clap `ValueEnum` 和 serde/string code enum 使用的代码枚举。
#[macro_export]
macro_rules! clap_code_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_enum! {
            #[derive(::clap::ValueEnum)]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }
    };
}

/// 带 snake_case serde、字符串转换、`Default` 和 hash 派生的代码类型。
#[macro_export]
macro_rules! serde_code_default {
    ($item:item) => {
        $crate::serde_code! {
            #[derive(Default)]
            $item
        }
    };
}

/// 带 snake_case serde、`Default`、变体列表和 `code` helper 的代码枚举。
#[macro_export]
macro_rules! serde_code_default_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_default! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
        $crate::__az_derive_aliases_code_default_enum_impl!($name);
    };
}

/// 面向需要 `Ord`/`PartialOrd` 和 hash 派生场景的有序代码类型。
#[macro_export]
macro_rules! serde_code_ord {
    ($item:item) => {
        $crate::serde_code! {
            #[derive(PartialOrd, Ord)]
            $item
        }
    };
}

/// 带 snake_case serde、`Default`、自定义 `Display` 和 hash 派生的有序代码类型。
#[macro_export]
macro_rules! serde_code_default_ord_display {
    ($item:item) => {
        $crate::__az_derive_aliases_serde_code_with!(
            (Default, PartialOrd, Ord),
            ::derive_more::Display,
            "snake_case",
            $item
        );
    };
}

/// 带自定义 `Display` 和 hash 派生的有序代码类型。
#[macro_export]
macro_rules! serde_code_ord_display {
    ($item:item) => {
        $crate::__az_derive_aliases_serde_code_with!(
            (PartialOrd, Ord),
            ::derive_more::Display,
            "snake_case",
            $item
        );
    };
}

/// 带 snake_case serde、`Default`、自定义 `Display`、变体列表和 `code` helper 的有序代码枚举。
#[macro_export]
macro_rules! serde_code_default_ord_display_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_default_ord_display! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
        $crate::__az_derive_aliases_code_default_enum_impl!($name);
    };
}

/// 带 snake_case serde、变体列表和 `code` helper 的有序代码枚举。
#[macro_export]
macro_rules! serde_code_ord_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_ord! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带 snake_case serde、自定义 `Display`、变体列表和 `code` helper 的有序代码枚举。
#[macro_export]
macro_rules! serde_code_ord_display_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_ord_display! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// 带 `Default` 和 hash 派生的有序代码类型。
#[macro_export]
macro_rules! serde_code_default_ord {
    ($item:item) => {
        $crate::serde_code_default! {
            #[derive(PartialOrd, Ord)]
            $item
        }
    };
}

/// 带 `Default`、变体列表和 `code` helper 的有序代码枚举。
#[macro_export]
macro_rules! serde_code_default_ord_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::serde_code_default_ord! {
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        }

        $crate::__az_derive_aliases_code_enum_impl!($name);
        $crate::__az_derive_aliases_code_default_enum_impl!($name);
    };
}

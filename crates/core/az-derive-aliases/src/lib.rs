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

/// Deserializable response/input type with debug formatting.
#[macro_export]
macro_rules! deserialize_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::serde::Deserialize), $item);
    };
}

/// Deserializable response/input type with clone support and debug formatting.
#[macro_export]
macro_rules! deserialize_clone_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, ::serde::Deserialize), $item);
    };
}

/// Serializable request/output type with debug formatting.
#[macro_export]
macro_rules! serialize_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::serde::Serialize), $item);
    };
}

/// Serializable request/output type with clone support and debug formatting.
#[macro_export]
macro_rules! serialize_clone_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, ::serde::Serialize), $item);
    };
}

/// Serializable request/output type with equality and debug traits.
#[macro_export]
macro_rules! serialize_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, PartialEq, Eq, ::serde::Serialize),
            $item
        );
    };
}

/// Serializable small request/output type with `Copy`, equality, and debug traits.
#[macro_export]
macro_rules! serialize_copy_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Copy, Debug, PartialEq, Eq, ::serde::Serialize),
            $item
        );
    };
}

/// Serializable request/output type with partial equality and debug traits.
#[macro_export]
macro_rules! serialize_partial_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, PartialEq, ::serde::Serialize), $item);
    };
}

/// Deserializable response/input type with equality and debug traits.
#[macro_export]
macro_rules! deserialize_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, PartialEq, Eq, ::serde::Deserialize),
            $item
        );
    };
}

/// Deserializable response/input type with partial equality and debug traits.
#[macro_export]
macro_rules! deserialize_partial_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, PartialEq, ::serde::Deserialize), $item);
    };
}

/// Lightweight conversion type with `derive_more::From`, debug, and equality.
#[macro_export]
macro_rules! from_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, ::derive_more::From, PartialEq), $item);
    };
}

/// Lightweight conversion type with `derive_more::From` and full equality.
#[macro_export]
macro_rules! from_plain_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, ::derive_more::From, PartialEq, Eq),
            $item
        );
    };
}

/// Lightweight `Copy` conversion type with `derive_more::From` and full equality.
#[macro_export]
macro_rules! from_copy_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Copy, Debug, ::derive_more::From, PartialEq, Eq),
            $item
        );
    };
}

/// Lightweight conversion/value type with `derive_more::From` and `Display`.
#[macro_export]
macro_rules! from_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                ::derive_more::From,
                ::derive_more::Display,
                PartialEq
            ),
            $item
        );
    };
}

/// Comparable error type with `thiserror` plus common equality/debug traits.
#[macro_export]
macro_rules! error_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, ::thiserror::Error, PartialEq, Eq),
            $item
        );
    };
}

/// Small comparable error type with `thiserror`, `Copy`, and common equality/debug traits.
#[macro_export]
macro_rules! error_copy_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Copy, Debug, ::thiserror::Error, PartialEq, Eq),
            $item
        );
    };
}

/// Basic library error type with `thiserror` and debug formatting.
#[macro_export]
macro_rules! error {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::thiserror::Error), $item);
    };
}

/// Implements `From` by matching explicit source patterns to target expressions.
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

/// Serde-friendly data type with equality and debug traits.
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

/// Serde-friendly identity type with equality, debug, and hash traits.
#[macro_export]
macro_rules! serde_eq_hash {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// Serde-friendly identity type with equality, debug, hash, and `Display`.
#[macro_export]
macro_rules! serde_eq_hash_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// Serde-friendly ordered identity type with equality, debug, hash, and ordering traits.
#[macro_export]
macro_rules! serde_eq_hash_ord {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                PartialOrd,
                Ord,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// Serde-friendly ordered identity type with equality, debug, hash, ordering, and `Display`.
#[macro_export]
macro_rules! serde_eq_hash_ord_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                PartialOrd,
                Ord,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// SQLx row-mapped type with `FromRow`.
#[macro_export]
macro_rules! sqlx_from_row {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((::sqlx::FromRow), $item);
    };
}

/// Shaku component type with `Component`.
#[macro_export]
macro_rules! shaku_component {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Component), $item);
    };
}

/// Serde-friendly data type with equality traits and no generated `Debug`.
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

/// Serde-friendly data type with equality traits and redacted `Debug`.
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

/// Plain structural type with `Clone`, `Debug`, `Eq`, and `PartialEq`.
#[macro_export]
macro_rules! plain_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, Eq, PartialEq), $item);
    };
}

/// Plain structural type with `Clone` only.
#[macro_export]
macro_rules! plain_clone {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone), $item);
    };
}

/// Plain structural type with `Clone` and `Copy`.
#[macro_export]
macro_rules! plain_copy {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Copy), $item);
    };
}

/// Plain structural type with `Default` only.
#[macro_export]
macro_rules! plain_default {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Default), $item);
    };
}

/// Plain structural type with `Debug` only.
#[macro_export]
macro_rules! plain_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug), $item);
    };
}

/// Plain structural type with `Clone` and `Debug`.
#[macro_export]
macro_rules! plain_clone_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug), $item);
    };
}

/// Plain structural type with `Debug` and `Default`.
#[macro_export]
macro_rules! plain_default_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, Default), $item);
    };
}

/// Plain structural type with `Clone` and `Default`.
#[macro_export]
macro_rules! plain_default_clone {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Default), $item);
    };
}

/// Plain structural type with `Clone`, `Debug`, and `Default`.
#[macro_export]
macro_rules! plain_default_clone_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, Default), $item);
    };
}

/// Plain structural type with `Clone`, `Debug`, and `Display`.
#[macro_export]
macro_rules! plain_clone_debug_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, ::derive_more::Display), $item);
    };
}

/// Plain structural type with `Clone`, `Eq`, and `PartialEq` but no generated `Debug`.
#[macro_export]
macro_rules! plain_eq_no_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Eq, PartialEq), $item);
    };
}

/// Plain structural type with equality traits and redacted `Debug`.
#[macro_export]
macro_rules! plain_eq_redacted {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, ::derive_more::Debug, Eq, PartialEq), $item);
    };
}

/// Plain structural type with `Clone` and redacted `Debug`.
#[macro_export]
macro_rules! plain_clone_redacted {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, ::derive_more::Debug), $item);
    };
}

/// Plain structural type with `Clone`, `Debug`, `Eq`, `Hash`, and `PartialEq`.
#[macro_export]
macro_rules! plain_eq_hash {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, Eq, Hash, PartialEq), $item);
    };
}

/// Plain structural type with `Clone`, `Debug`, `Eq`, `Hash`, `PartialEq`, and `Display`.
#[macro_export]
macro_rules! plain_eq_hash_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, Eq, Hash, PartialEq, ::derive_more::Display),
            $item
        );
    };
}

/// Plain structural type with `Clone`, `Debug`, and `PartialEq`.
#[macro_export]
macro_rules! plain_partial_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, PartialEq), $item);
    };
}

/// Plain structural type with `Clone`, `Debug`, `PartialEq`, and `Display`.
#[macro_export]
macro_rules! plain_partial_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, PartialEq, ::derive_more::Display),
            $item
        );
    };
}

/// Plain structural type with `Clone`, `Debug`, `Default`, `Eq`, and `PartialEq`.
#[macro_export]
macro_rules! plain_default_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, Default, Eq, PartialEq), $item);
    };
}

/// Plain structural type with `Clone`, `Debug`, `Default`, and `PartialEq`.
#[macro_export]
macro_rules! plain_default_partial_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, Default, PartialEq), $item);
    };
}

/// Plain structural type with `Clone`, `Copy`, `Debug`, `Eq`, and `PartialEq`.
#[macro_export]
macro_rules! plain_copy_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Copy, Debug, Eq, PartialEq), $item);
    };
}

/// Plain structural type with `Clone`, `Copy`, `Debug`, `Eq`, `PartialEq`, and `Display`.
#[macro_export]
macro_rules! plain_copy_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Copy, Debug, Eq, PartialEq, ::derive_more::Display),
            $item
        );
    };
}

/// Plain structural type with `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, and `PartialEq`.
#[macro_export]
macro_rules! plain_copy_eq_hash {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Copy, Debug, Eq, Hash, PartialEq), $item);
    };
}

/// Plain structural type with `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `PartialEq`, `PartialOrd`,
/// `Ord`, and `Display`.
#[macro_export]
macro_rules! plain_copy_eq_hash_ord_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Eq,
                Hash,
                PartialEq,
                PartialOrd,
                Ord,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// Plain structural type with `Clone`, `Debug`, `Eq`, `Hash`, `PartialEq`, `PartialOrd`, `Ord`,
/// and `Display`.
#[macro_export]
macro_rules! plain_eq_hash_ord_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                Eq,
                Hash,
                PartialEq,
                PartialOrd,
                Ord,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// Plain structural type with `Clone`, `Copy`, `Debug`, `Eq`, `Hash`, `PartialEq`, and `Display`.
#[macro_export]
macro_rules! plain_copy_eq_hash_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Eq,
                Hash,
                PartialEq,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// Plain structural type with `Clone`, `Debug`, `Eq`, `PartialEq`, and `Display`.
#[macro_export]
macro_rules! plain_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, Eq, PartialEq, ::derive_more::Display),
            $item
        );
    };
}

/// Plain structural type with `Clone`, `Copy`, `Debug`, `Default`, `Eq`, and `PartialEq`.
#[macro_export]
macro_rules! plain_default_copy_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Copy, Debug, Default, Eq, PartialEq), $item);
    };
}

/// Plain structural type with `Clone`, `Copy`, `Debug`, `Default`, `Eq`, `PartialEq`, and `Display`.
#[macro_export]
macro_rules! plain_default_copy_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Default,
                Eq,
                PartialEq,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// Clap parser type with `Debug` and `Parser`.
#[macro_export]
macro_rules! clap_parser {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::clap::Parser), $item);
    };
}

/// Clap args type with `Debug` and `Args`.
#[macro_export]
macro_rules! clap_args {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::clap::Args), $item);
    };
}

/// Clap args type with `Debug`, `Args`, `Clone`, and `Default`.
#[macro_export]
macro_rules! clap_args_default_clone {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::clap::Args, Clone, Default), $item);
    };
}

/// Clap subcommand type with `Debug` and `Subcommand`.
#[macro_export]
macro_rules! clap_subcommand {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::clap::Subcommand), $item);
    };
}

/// Dioxus props type with `Clone` and `PartialEq`.
#[macro_export]
macro_rules! dioxus_props {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((::dioxus::prelude::Props, Clone, PartialEq), $item);
    };
}

/// Clap value enum with `Debug`, `Clone`, `Copy`, and equality traits.
#[macro_export]
macro_rules! clap_value_enum {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Debug, Clone, Copy, PartialEq, Eq, ::clap::ValueEnum),
            $item
        );
    };
}

/// SeaORM entity model with `Clone`, `Debug`, and `PartialEq`.
#[macro_export]
macro_rules! seaorm_entity_model {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug, PartialEq, DeriveEntityModel), $item);
    };
}

/// SeaORM entity model with `Clone`, `Debug`, `PartialEq`, and `Eq`.
#[macro_export]
macro_rules! seaorm_entity_model_eq {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (Clone, Debug, PartialEq, Eq, DeriveEntityModel),
            $item
        );
    };
}

/// SeaORM relation enum with the standard iterator and debug traits.
#[macro_export]
macro_rules! seaorm_relation {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Copy, Clone, Debug, EnumIter, DeriveRelation), $item);
    };
}

/// Serde-friendly small data type with equality, debug, and `Copy`.
#[macro_export]
macro_rules! serde_eq_copy {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
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

/// Serde-friendly small data type with equality, debug, `Copy`, and `Display`.
#[macro_export]
macro_rules! serde_eq_copy_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// `Copy` value wrapper with `derive_more::From`, debug, equality, and `Display`.
#[macro_export]
macro_rules! from_copy_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                ::derive_more::From,
                ::derive_more::Display,
                PartialEq,
                Eq
            ),
            $item
        );
    };
}

/// Serde-friendly small data type with equality, debug, `Copy`, `Default`, and hash traits.
#[macro_export]
macro_rules! serde_eq_default_copy {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Default,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// Ordered serde-friendly small data type with equality, debug, `Copy`, `Default`, and hash traits.
#[macro_export]
macro_rules! serde_eq_default_copy_ord {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Default,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                PartialOrd,
                Ord,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// Serde-friendly data type with equality, debug, and `Default`.
#[macro_export]
macro_rules! serde_eq_default {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                Default,
                PartialEq,
                Eq,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// Serde-friendly data type with partial equality and debug traits.
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

/// Serde-friendly data type with partial equality, debug, and `Default`.
#[macro_export]
macro_rules! serde_partial_eq_default {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                Default,
                PartialEq,
                ::serde::Serialize,
                ::serde::Deserialize
            ),
            $item
        );
    };
}

/// Serde-friendly data type with partial equality, debug, and `Display`.
#[macro_export]
macro_rules! serde_partial_eq_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Debug,
                PartialEq,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::derive_more::Display
            ),
            $item
        );
    };
}

/// Code-backed data type with snake_case serde, string conversion, and hash derives.
#[macro_export]
macro_rules! serde_code {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::strum::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
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

/// Code-backed data type with kebab-case serde, string conversion, and hash derives.
#[macro_export]
macro_rules! serde_kebab_code {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::strum::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "kebab-case")]
            #[strum(serialize_all = "kebab-case")]
            $item
        );
    };
}

/// Code-backed data type with lowercase serde, string conversion, and hash derives.
#[macro_export]
macro_rules! serde_lower_code {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::strum::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "lowercase")]
            #[strum(serialize_all = "lowercase")]
            $item
        );
    };
}

/// Code-backed data type with snake_case serde, custom `Display`, and hash derives.
#[macro_export]
macro_rules! serde_code_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::derive_more::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
            $item
        );
    };
}

/// Code-backed enum with lowercase serde, string conversion, variant list, and `code` helpers.
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

/// Code-backed enum with snake_case serde, string conversion, variant list, and `code` helpers.
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

/// Code-backed enum with kebab-case serde, string conversion, variant list, and `code` helpers.
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

/// Code-backed enum with snake_case serde, custom `Display`, variant list, and `code` helpers.
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

/// Plain code-backed enum with snake_case string conversion, custom `Display`, and variant list.
#[macro_export]
macro_rules! plain_code_display_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Default,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::derive_more::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[strum(serialize_all = "snake_case")]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        );

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// Plain code-backed enum with explicit string conversion and variant list.
#[macro_export]
macro_rules! plain_code_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Eq,
                PartialEq,
                ::core::hash::Hash,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        );

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// Plain code-backed enum with custom `Display`, explicit string conversion, and variant list.
#[macro_export]
macro_rules! plain_code_display_no_default_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::derive_more::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[strum(serialize_all = "snake_case")]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        );

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// Code-backed enum usable as both a Clap `ValueEnum` and a serde/string code enum.
#[macro_export]
macro_rules! clap_code_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $($body:tt)*
        }
    ) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::strum::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray,
                ::clap::ValueEnum
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
            $(#[$meta])*
            $vis enum $name {
                $($body)*
            }
        );

        $crate::__az_derive_aliases_code_enum_impl!($name);
    };
}

/// Code-backed data type with snake_case serde, string conversion, `Default`, and hash derives.
#[macro_export]
macro_rules! serde_code_default {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Default,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::strum::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
            $item
        );
    };
}

/// Code-backed enum with snake_case serde, `Default`, variant list, and `code` helpers.
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
    };
}

/// Ordered code-backed data type for cases that need `Ord`/`PartialOrd` and hash derives.
#[macro_export]
macro_rules! serde_code_ord {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                PartialOrd,
                Ord,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::strum::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
            $item
        );
    };
}

/// Ordered code-backed data type with snake_case serde, `Default`, custom `Display`, and hash derives.
#[macro_export]
macro_rules! serde_code_default_ord_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Default,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                PartialOrd,
                Ord,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::derive_more::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
            $item
        );
    };
}

/// Ordered code-backed data type with custom `Display` and hash derives.
#[macro_export]
macro_rules! serde_code_ord_display {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                PartialOrd,
                Ord,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::derive_more::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
            $item
        );
    };
}

/// Ordered code-backed enum with snake_case serde, `Default`, custom `Display`, variant list, and `code` helpers.
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
    };
}

/// Ordered code-backed enum with snake_case serde, variant list, and `code` helpers.
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

/// Ordered code-backed enum with snake_case serde, custom `Display`, variant list, and `code` helpers.
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

/// Ordered code-backed data type with `Default` and hash derives.
#[macro_export]
macro_rules! serde_code_default_ord {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!(
            (
                Clone,
                Copy,
                Debug,
                Default,
                PartialEq,
                Eq,
                ::core::hash::Hash,
                PartialOrd,
                Ord,
                ::serde::Serialize,
                ::serde::Deserialize,
                ::strum::Display,
                ::strum::EnumString,
                ::strum::IntoStaticStr,
                ::strum::VariantArray
            ),
            #[serde(rename_all = "snake_case")]
            #[strum(serialize_all = "snake_case")]
            $item
        );
    };
}

/// Ordered code-backed enum with `Default`, variant list, and `code` helpers.
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
    };
}

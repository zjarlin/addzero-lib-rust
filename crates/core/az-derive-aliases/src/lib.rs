#![doc = include_str!("../README.md")]

pub use macro_rules_attribute::apply;

/// Comparable error type with `thiserror` plus common equality/debug traits.
#[macro_export]
macro_rules! error_eq {
    ($item:item) => {
        #[derive(Clone, Debug, ::thiserror::Error, PartialEq, Eq)]
        $item
    };
}

/// Serde-friendly data type with equality and debug traits.
#[macro_export]
macro_rules! serde_eq {
    ($item:item) => {
        #[derive(Clone, Debug, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize)]
        $item
    };
}

/// Serde-friendly data type with equality, debug, and `Default`.
#[macro_export]
macro_rules! serde_eq_default {
    ($item:item) => {
        #[derive(
            Clone, Debug, Default, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize,
        )]
        $item
    };
}

/// Code-backed data type with snake_case serde and string conversion derives.
#[macro_export]
macro_rules! serde_code {
    ($item:item) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::strum::EnumString,
            ::strum::IntoStaticStr,
            ::strum::VariantArray,
        )]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        $item
    };
}

/// Code-backed data type with snake_case serde, string conversion, and `Default`.
#[macro_export]
macro_rules! serde_code_default {
    ($item:item) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            Default,
            PartialEq,
            Eq,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::strum::EnumString,
            ::strum::IntoStaticStr,
            ::strum::VariantArray,
        )]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        $item
    };
}

/// Ordered code-backed data type for cases that need `Ord`/`PartialOrd`.
#[macro_export]
macro_rules! serde_code_ord {
    ($item:item) => {
        #[derive(
            Clone,
            Copy,
            Debug,
            PartialEq,
            Eq,
            PartialOrd,
            Ord,
            ::serde::Serialize,
            ::serde::Deserialize,
            ::strum::EnumString,
            ::strum::IntoStaticStr,
            ::strum::VariantArray,
        )]
        #[serde(rename_all = "snake_case")]
        #[strum(serialize_all = "snake_case")]
        $item
    };
}

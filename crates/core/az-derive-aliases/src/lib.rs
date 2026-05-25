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

/// Serializable request/output type with debug formatting.
#[macro_export]
macro_rules! serialize_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Debug, ::serde::Serialize), $item);
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

/// Plain structural type with `Clone` and `Debug`.
#[macro_export]
macro_rules! plain_clone_debug {
    ($item:item) => {
        $crate::__az_derive_aliases_derive!((Clone, Debug), $item);
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

        impl $name {
            pub const ALL: &'static [Self] = <Self as ::strum::VariantArray>::VARIANTS;

            #[must_use]
            pub fn code(self) -> &'static str {
                self.into()
            }

            pub fn from_code(value: &str) -> Option<Self> {
                value.parse().ok()
            }
        }
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

        impl $name {
            pub const ALL: &'static [Self] = <Self as ::strum::VariantArray>::VARIANTS;

            #[must_use]
            pub fn code(self) -> &'static str {
                self.into()
            }

            pub fn from_code(value: &str) -> Option<Self> {
                value.parse().ok()
            }
        }
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

        impl $name {
            pub const ALL: &'static [Self] = <Self as ::strum::VariantArray>::VARIANTS;

            #[must_use]
            pub fn code(self) -> &'static str {
                self.into()
            }

            pub fn from_code(value: &str) -> Option<Self> {
                value.parse().ok()
            }
        }
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

        impl $name {
            pub const ALL: &'static [Self] = <Self as ::strum::VariantArray>::VARIANTS;

            #[must_use]
            pub fn code(self) -> &'static str {
                self.into()
            }

            pub fn from_code(value: &str) -> Option<Self> {
                value.parse().ok()
            }
        }
    };
}

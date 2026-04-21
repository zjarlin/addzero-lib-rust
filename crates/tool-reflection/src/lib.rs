mod cache;
mod metainfo;
mod value;

pub use cache::ExpiringCache;
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

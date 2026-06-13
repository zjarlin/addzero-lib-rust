//! Declarative entity macro entrypoint.

/// 声明一个 Jimmer 风格实体模块。
///
/// 它会生成实体 marker 类型、实体元数据、表对象，以及每个字段的强类型访问函数。
///
/// ```
/// use rimmer::{entity, metadata::FieldKind};
///
/// entity! {
///     pub mod author {
///         pub struct Author => "AUTHOR" {
///             id id: i64 => "ID",
///             key first_name: String => "FIRST_NAME",
///             scalar last_name: String => "LAST_NAME",
///         }
///     }
/// }
///
/// assert_eq!(author::entity().table_name(), "AUTHOR");
/// assert_eq!(author::first_name().kind(), FieldKind::Key);
/// ```
#[macro_export]
macro_rules! entity {
    (
        $module_vis:vis mod $module:ident {
            $entity_vis:vis struct $entity:ident => $table:literal {
                $(
                    $kind:ident $field:ident : $ty:ty => $column:literal
                ),* $(,)?
            }
        }
    ) => {
        $module_vis mod $module {
            #[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
            $entity_vis struct $entity;

            static FIELDS: &[$crate::metadata::FieldMetadata] = &[
                $(
                    $crate::metadata::FieldMetadata::new(
                        stringify!($field),
                        $column,
                        $crate::entity!(@kind $kind),
                    ),
                )*
            ];

            /// 返回当前实体的元模型定义。
            pub fn entity() -> $crate::metadata::EntityDef<$entity> {
                $crate::metadata::EntityDef::new(stringify!($entity), $table, FIELDS)
            }

            /// 返回当前实体的表对象。
            pub fn table() -> $crate::metadata::Table<$entity> {
                $crate::metadata::Table::new(entity())
            }

            $(
                #[doc = concat!("返回字段 `", stringify!($field), "` 的强类型表达式。")]
                pub fn $field() -> $crate::expression::Field<$entity, $ty> {
                    $crate::expression::Field::new(
                        entity(),
                        stringify!($field),
                        $column,
                        $crate::entity!(@kind $kind),
                    )
                }
            )*

            impl $crate::metadata::Entity for $entity {
                fn entity() -> $crate::metadata::EntityDef<Self> {
                    entity()
                }
            }
        }
    };
    (@kind id) => {
        $crate::metadata::FieldKind::Id
    };
    (@kind key) => {
        $crate::metadata::FieldKind::Key
    };
    (@kind scalar) => {
        $crate::metadata::FieldKind::Scalar
    };
    (@kind many_to_one) => {
        $crate::metadata::FieldKind::ManyToOne
    };
    (@kind one_to_many) => {
        $crate::metadata::FieldKind::OneToMany
    };
    (@kind many_to_many) => {
        $crate::metadata::FieldKind::ManyToMany
    };
    (@kind transient) => {
        $crate::metadata::FieldKind::Transient
    };
    (@kind id_view) => {
        $crate::metadata::FieldKind::IdView
    };
}

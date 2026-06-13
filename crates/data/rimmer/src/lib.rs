//! Jimmer 风格的 Rust ORM 核心门面。
//!
//! 这个 crate 的目标不是逐行移植 JVM Jimmer，而是先把最重要的使用体验固定下来：
//! 强类型字段、动态谓词、Fetcher 返回形状，以及能区分“未指定字段”和 `NULL`
//! 的 draft 保存模型。Fetcher 使用 JSON 这类通用格式承载形状，不引入 DTO language。
//! 当前已经提供 SQL plan 和基于 `sqlx::AnyPool` 的基础执行器。

#![forbid(unsafe_code)]

automod::dir!("src");

pub use dialect::SqlDialect;
pub use draft::{
    Draft, DraftCollection, DraftCreator, DraftField, ErasedDraft, ErasedEntityDef, new_draft,
};
pub use error::{OrmError, OrmResult};
pub use executor::{
    JsonQueryResult, SaveExecution, SqlxJimmerClient, SqlxQueryBuilder, SqlxSaveCommand,
};
pub use expression::{CollectionRelation, Field, IntoPredicate, Order, Predicate};
pub use fetcher::{
    CollectionFetchOptions, FetchField, FetchJoinTable, FetchRelation, FetchShape, Fetcher,
    FetcherBuilder, FetcherCreator, ManyToManyJoin, new_fetcher,
};
pub use metadata::{Entity, EntityDef, FieldKind, FieldMetadata, Table};
pub use query::{JimmerClient, QueryBuilder, QueryBuilderExt, QueryPlan, Selection};
pub use rimmer_macros::Entity;
pub use save::{SaveCommand, SaveMode, SavePlan};
pub use value::{ScalarValue, ToScalarValue};

/// 声明一个 Jimmer 风格实体模块。
///
/// 它会生成实体 marker 类型、实体元数据、表对象，以及每个字段的强类型访问函数。
///
/// ```
/// use rimmer::{entity, FieldKind};
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

            static FIELDS: &[$crate::FieldMetadata] = &[
                $(
                    $crate::FieldMetadata::new(
                        stringify!($field),
                        $column,
                        $crate::entity!(@kind $kind),
                    ),
                )*
            ];

            /// 返回当前实体的元模型定义。
            pub fn entity() -> $crate::EntityDef<$entity> {
                $crate::EntityDef::new(stringify!($entity), $table, FIELDS)
            }

            /// 返回当前实体的表对象。
            pub fn table() -> $crate::Table<$entity> {
                $crate::Table::new(entity())
            }

            $(
                #[doc = concat!("返回字段 `", stringify!($field), "` 的强类型表达式。")]
                pub fn $field() -> $crate::Field<$entity, $ty> {
                    $crate::Field::new(
                        entity(),
                        stringify!($field),
                        $column,
                        $crate::entity!(@kind $kind),
                    )
                }
            )*

            impl $crate::Entity for $entity {
                fn entity() -> $crate::EntityDef<Self> {
                    entity()
                }
            }
        }
    };
    (@kind id) => {
        $crate::FieldKind::Id
    };
    (@kind key) => {
        $crate::FieldKind::Key
    };
    (@kind scalar) => {
        $crate::FieldKind::Scalar
    };
    (@kind many_to_one) => {
        $crate::FieldKind::ManyToOne
    };
    (@kind one_to_many) => {
        $crate::FieldKind::OneToMany
    };
    (@kind many_to_many) => {
        $crate::FieldKind::ManyToMany
    };
    (@kind transient) => {
        $crate::FieldKind::Transient
    };
    (@kind id_view) => {
        $crate::FieldKind::IdView
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    entity! {
        pub mod book_store {
            pub struct BookStore => "BOOK_STORE" {
                id id: i64 => "ID",
                key name: String => "NAME",
                scalar website: Option<String> => "WEBSITE",
            }
        }
    }

    entity! {
        pub mod book {
            pub struct Book => "BOOK" {
                id id: i64 => "ID",
                key name: String => "NAME",
                key edition: i32 => "EDITION",
                scalar price: f64 => "PRICE",
                many_to_one store_id: Option<i64> => "STORE_ID",
            }
        }
    }

    #[test]
    fn generated_entity_api_should_expose_table_and_fields() {
        let table = book_store::table();
        let id = book::id();
        let store_id = book::store_id();

        // 断言宏生成的表和字段 API 能稳定暴露给业务代码。
        assert_eq!(table.entity().table_name(), "BOOK_STORE");
        assert_eq!(id.column_name(), "ID");
        assert_eq!(store_id.kind(), FieldKind::ManyToOne);
    }

    #[test]
    fn query_should_keep_jimmer_like_shape() {
        let name = Some("GraphQL in Action");
        let fetcher = new_fetcher(book::entity()).by(|f| {
            f.field(book::name()).field(book::edition()).association(
                "store",
                new_fetcher(book_store::entity()).by(|store| store.field(book_store::name())),
            )
        });

        let plan = JimmerClient::new()
            .create_query(book::table())
            .where_(book::name().eq_if_not_blank(name))
            .order_by(book::edition().desc())
            .select(book::table().fetch(fetcher))
            .build()
            .unwrap();

        // 断言查询 API 能像 Jimmer 一样按 Fetcher 选择根实体列。
        assert_eq!(
            plan.sql,
            concat!(
                r#"SELECT "BOOK"."ID", "BOOK"."NAME", "BOOK"."EDITION" "#,
                r#"FROM "BOOK" "#,
                r#"WHERE "BOOK"."NAME" = ? "#,
                r#"ORDER BY "BOOK"."EDITION" DESC"#
            )
        );
        assert_eq!(
            plan.params,
            vec![ScalarValue::Text("GraphQL in Action".into())]
        );
    }

    #[test]
    fn dynamic_predicate_should_ignore_absent_value() {
        let plan = JimmerClient::new()
            .create_query(book::table())
            .where_(book::name().eq_if_not_blank(Some("")))
            .where_(book::price().between_if(None::<f64>, Some(100.0)))
            .select(book::table().fetch(new_fetcher(book::entity()).by(|f| f.field(book::name()))))
            .build()
            .unwrap();

        // 断言动态谓词为空时不会污染 WHERE。
        assert_eq!(plan.sql, r#"SELECT "BOOK"."ID", "BOOK"."NAME" FROM "BOOK""#);
        assert!(plan.params.is_empty());
    }

    #[test]
    fn fetcher_shape_should_round_trip_as_json() {
        let fetcher = new_fetcher(book::entity()).by(|f| {
            f.field(book::name()).field(book::edition()).association(
                "store",
                new_fetcher(book_store::entity()).by(|store| store.field(book_store::name())),
            )
        });
        let json = fetcher.to_json().unwrap();
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();

        // 断言 Fetcher 不依赖自定义 DTO 语言，而是落到通用 JSON 形状。
        assert_eq!(value["entityName"], "Book");
        assert_eq!(value["fields"][0]["kind"], "key");

        let restored: Fetcher<book::Book> = Fetcher::from_json(book::entity(), &json).unwrap();

        // 断言 JSON 形状能重新恢复为类型化 Fetcher。
        assert_eq!(restored.shape(), fetcher.shape());
    }

    #[test]
    fn save_should_distinguish_null_from_unspecified() {
        let draft = new_draft(book_store::entity()).by(|d| {
            d.set(book_store::id(), 1_i64)
                .set(book_store::name(), "O'REILLY+")
                .set_null(book_store::website())
        });

        let plan = JimmerClient::new()
            .save(draft)
            .set_mode(SaveMode::UpdateOnly)
            .build()
            .unwrap();

        // 断言显式 null 会进入更新字段，未指定字段则不会出现。
        assert_eq!(
            plan.sql,
            r#"UPDATE "BOOK_STORE" SET "NAME" = ?, "WEBSITE" = ? WHERE "ID" = ?"#
        );
        assert_eq!(
            plan.params,
            vec![
                ScalarValue::Text("O'REILLY+".into()),
                ScalarValue::Null,
                ScalarValue::I64(1)
            ]
        );
    }
}

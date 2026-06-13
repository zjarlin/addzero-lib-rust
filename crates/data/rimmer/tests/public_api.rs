use rimmer::draft::new_draft;
use rimmer::entity;
use rimmer::fetcher::{Fetcher, new_fetcher};
use rimmer::metadata::FieldKind;
use rimmer::query::{JimmerClient, QueryBuilderExt};
use rimmer::save::SaveMode;
use rimmer::value::ScalarValue;

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

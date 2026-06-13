use rimmer::fetcher::{CollectionFetchOptions, Fetcher};
use rimmer::query::{JimmerClient, QueryBuilderExt};
use rimmer::value::ScalarValue;

#[derive(rimmer::derive::Entity)]
#[rimmer(table = "BOOK_STORE")]
pub struct BookStore {
    #[rimmer(id, column = "ID")]
    pub id: i64,
    #[rimmer(key, column = "NAME")]
    pub name: String,
    #[rimmer(column = "WEBSITE")]
    pub website: Option<String>,
}

#[derive(rimmer::derive::Entity)]
#[rimmer(table = "BOOK")]
pub struct Book {
    #[rimmer(id, column = "ID")]
    pub id: i64,
    #[rimmer(key, column = "NAME")]
    pub name: String,
    #[rimmer(key, column = "EDITION")]
    pub edition: i32,
    #[rimmer(column = "PRICE")]
    pub price: f64,
    #[rimmer(many_to_one, column = "STORE_ID")]
    pub store_id: Option<i64>,
}

#[test]
fn derive_entity_should_generate_jimmer_style_table_and_fields() {
    let fetcher = Book::fetcher().by(|book| {
        book.field(Book::name()).field(Book::edition()).association(
            "store",
            BookStore::fetcher().by(|store| store.field(BookStore::name())),
        )
    });
    let plan = JimmerClient::new()
        .create_query(Book::table())
        .where_(Book::name().eq_if_not_blank(Some("GraphQL in Action")))
        .order_by(Book::edition().desc())
        .select(Book::table().fetch(fetcher))
        .build()
        .unwrap();

    // 断言 derive 生成的 API 能维持 Jimmer 风格查询链。
    assert_eq!(
        plan.sql,
        concat!(
            r#"SELECT "BOOK"."ID", "BOOK"."NAME", "BOOK"."EDITION" "#,
            r#"FROM "BOOK" "#,
            r#"WHERE "BOOK"."NAME" = ? "#,
            r#"ORDER BY "BOOK"."EDITION" DESC"#
        )
    );
}

#[test]
fn derive_entity_should_generate_many_to_one_join_query() {
    let fetcher = Book::fetcher().by(|book| {
        book.field(Book::name()).many_to_one(
            "store",
            Book::store_id(),
            BookStore::id(),
            BookStore::fetcher().by(|store| store.field(BookStore::name())),
        )
    });
    let plan = JimmerClient::new()
        .create_query(Book::table())
        .where_(Book::name().eq_if_not_blank(Some("GraphQL in Action")))
        .select(Book::table().fetch(fetcher))
        .build()
        .unwrap();

    // 断言 many-to-one Fetcher 会驱动自动 LEFT JOIN。
    assert_eq!(
        plan.sql,
        concat!(
            r#"SELECT "BOOK"."ID" AS "__rimmer__root__ID", "#,
            r#""BOOK"."NAME" AS "__rimmer__root__NAME", "#,
            r#""__rimmer_join__store"."ID" AS "__rimmer__store__ID", "#,
            r#""__rimmer_join__store"."NAME" AS "__rimmer__store__NAME" "#,
            r#"FROM "BOOK" "#,
            r#"LEFT JOIN "BOOK_STORE" "__rimmer_join__store" "#,
            r#"ON "BOOK"."STORE_ID" = "__rimmer_join__store"."ID" "#,
            r#"WHERE "BOOK"."NAME" = ?"#
        )
    );
}

#[test]
fn derive_entity_should_generate_one_to_many_exists_query() {
    let books = BookStore::id().one_to_many(Book::store_id());
    let plan = JimmerClient::new()
        .create_query(BookStore::table())
        .where_(
            books.exists(
                Book::name()
                    .eq("GraphQL in Action")
                    .and(Book::edition().eq(1_i32)),
            ),
        )
        .select(
            BookStore::table()
                .fetch(BookStore::fetcher().by(|store| store.field(BookStore::name()))),
        )
        .build()
        .unwrap();

    // 断言集合关联谓词会生成 Jimmer 风格 EXISTS 隐式子查询。
    assert_eq!(
        plan.sql,
        concat!(
            r#"SELECT "BOOK_STORE"."ID", "BOOK_STORE"."NAME" FROM "BOOK_STORE" "#,
            r#"WHERE EXISTS (SELECT 1 FROM "BOOK" "#,
            r#"WHERE "BOOK"."STORE_ID" = "BOOK_STORE"."ID" "#,
            r#"AND ("BOOK"."NAME" = ?) AND ("BOOK"."EDITION" = ?))"#
        )
    );
    assert_eq!(
        plan.params,
        vec![
            ScalarValue::Text("GraphQL in Action".into()),
            ScalarValue::I64(1),
        ]
    );
}

#[test]
fn derive_entity_should_generate_json_fetcher_shape() {
    let fetcher = Book::fetcher().by(|book| {
        book.field(Book::name()).association(
            "store",
            BookStore::fetcher().by(|store| store.field(BookStore::name())),
        )
    });
    let json = fetcher.to_json().unwrap();
    let restored: Fetcher<Book> = Fetcher::from_json(Book::entity(), &json).unwrap();
    let value = fetcher.to_json_value().unwrap();
    let restored_from_value: Fetcher<Book> =
        Fetcher::from_json_value(Book::entity(), value).unwrap();

    // 断言 derive API 生成的 Fetcher 仍然使用通用 JSON 形状。
    assert_eq!(restored.shape(), fetcher.shape());
    assert_eq!(restored_from_value.shape(), fetcher.shape());
}

#[test]
fn fetcher_from_json_should_reject_root_field_drift() {
    let fetcher = Book::fetcher().by(|book| book.field(Book::name()));
    let mut value = fetcher.to_json_value().unwrap();
    value["fields"][0]["columnName"] = serde_json::Value::String("TITLE".to_string());

    let error = Fetcher::<Book>::from_json_value(Book::entity(), value).unwrap_err();

    // 断言持久化 Fetcher 形状不能绕过实体字段元模型。
    assert!(error.to_string().contains("invalid fetcher shape"));
}

#[test]
fn fetcher_from_json_should_reject_relation_metadata_drift() {
    let fetcher = Book::fetcher().by(|book| {
        book.many_to_one(
            "store",
            Book::store_id(),
            BookStore::id(),
            BookStore::fetcher().by(|store| store.field(BookStore::name())),
        )
    });
    let mut value = fetcher.to_json_value().unwrap();
    value["fields"][0]["relation"]["sourceColumn"] =
        serde_json::Value::String("UNKNOWN_STORE_ID".to_string());

    let error = Fetcher::<Book>::from_json_value(Book::entity(), value).unwrap_err();

    // 断言关联 Fetcher 形状必须继续引用根实体真实列。
    assert!(error.to_string().contains("invalid fetcher shape"));
}

#[test]
fn derive_entity_should_generate_json_collection_fetcher_options() {
    let fetcher = BookStore::fetcher().by(|store| {
        store.field(BookStore::name()).one_to_many_with_options(
            "books",
            BookStore::id(),
            Book::store_id(),
            Book::fetcher().by(|book| book.field(Book::name())),
            CollectionFetchOptions::new()
                .filter(Book::edition().eq(1_i32))
                .order_by(Book::name().desc())
                .limit(2),
        )
    });
    let json = fetcher.to_json().unwrap();
    let restored: Fetcher<BookStore> = Fetcher::from_json(BookStore::entity(), &json).unwrap();

    // 断言集合关联级配置会进入通用 JSON Fetcher 形状。
    assert_eq!(restored.shape(), fetcher.shape());
    assert!(json.contains(r#""collectionOptions""#));
    assert!(json.contains(r#""limit":2"#));
}

#[test]
fn derive_entity_should_generate_draft_entry() {
    let draft = BookStore::draft(|store| {
        store
            .set(BookStore::id(), 1_i64)
            .set(BookStore::name(), "O'REILLY+")
            .set_null(BookStore::website())
    });
    let plan = JimmerClient::new()
        .save(draft)
        .set_mode(rimmer::save::SaveMode::UpdateOnly)
        .build()
        .unwrap();

    // 断言 derive 生成的 Draft 入口保留“未指定不更新，显式 null 更新”的语义。
    assert_eq!(
        plan.params,
        vec![
            ScalarValue::Text("O'REILLY+".into()),
            ScalarValue::Null,
            ScalarValue::I64(1),
        ]
    );
}

#[test]
fn derive_entity_should_generate_one_to_many_save_graph_plan() {
    let draft = BookStore::draft(|store| {
        store.set(BookStore::id(), 1_i64).one_to_many(
            "books",
            BookStore::id(),
            Book::store_id(),
            vec![Book::draft(|book| {
                book.set(Book::id(), 10_i64)
                    .set(Book::name(), "GraphQL in Action")
                    .set(Book::edition(), 1_i32)
            })],
        )
    });
    let plan = JimmerClient::new()
        .save(draft)
        .set_mode(rimmer::save::SaveMode::InsertOnly)
        .build()
        .unwrap();

    // 断言保存对象图会为子对象自动补父外键。
    assert_eq!(plan.children.len(), 1);
    assert_eq!(
        plan.children[0].params,
        vec![
            ScalarValue::I64(10),
            ScalarValue::Text("GraphQL in Action".into()),
            ScalarValue::I64(1),
            ScalarValue::I64(1),
        ]
    );
}

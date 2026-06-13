use rimmer::dialect::SqlDialect;
use rimmer::executor::SqlxJimmerClient;
use rimmer::fetcher::{CollectionFetchOptions, ManyToManyJoin};
use rimmer::query::{JimmerClient, QueryBuilderExt};
use serde_json::json;
use sqlx::any::AnyPoolOptions;

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
    #[rimmer(many_to_one, column = "STORE_ID")]
    pub store_id: Option<i64>,
}

#[derive(rimmer::derive::Entity)]
#[rimmer(table = "AUTHOR")]
pub struct Author {
    #[rimmer(id, column = "ID")]
    pub id: i64,
    #[rimmer(key, column = "FIRST_NAME")]
    pub first_name: String,
    #[rimmer(key, column = "LAST_NAME")]
    pub last_name: String,
}

#[tokio::test]
async fn sqlx_client_should_execute_query_as_json() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;

    // 断言内存库执行路径使用 SQLite 问号占位符方言。
    assert_eq!(client.dialect(), SqlDialect::Sqlite);

    let fetcher =
        BookStore::fetcher().by(|store| store.field(BookStore::name()).field(BookStore::website()));
    let result = client
        .create_query(BookStore::table())
        .where_(BookStore::name().eq_if_not_blank(Some("O'REILLY")))
        .select(BookStore::table().fetch(fetcher))
        .execute_json()
        .await
        .unwrap();

    // 断言运行时查询能返回 Fetcher 对应的 JSON 行。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 1,
            "NAME": "O'REILLY",
            "WEBSITE": "https://www.oreilly.com"
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_many_to_one_fetcher_as_nested_json() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    seed_book(client.pool()).await;

    let fetcher = Book::fetcher().by(|book| {
        book.field(Book::name()).many_to_one(
            "store",
            Book::store_id(),
            BookStore::id(),
            BookStore::fetcher().by(|store| store.field(BookStore::name())),
        )
    });
    let result = client
        .create_query(Book::table())
        .where_(Book::name().eq_if_not_blank(Some("GraphQL in Action")))
        .select(Book::table().fetch(fetcher))
        .execute_json()
        .await
        .unwrap();

    // 断言 Fetcher 的 many-to-one 关联会被装配为嵌套 JSON 对象。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 10,
            "NAME": "GraphQL in Action",
            "store": {
                "ID": 1,
                "NAME": "O'REILLY"
            }
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_one_to_many_fetcher_as_nested_json_array() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    seed_book(client.pool()).await;
    seed_second_book(client.pool()).await;

    let fetcher = BookStore::fetcher().by(|store| {
        store.field(BookStore::name()).one_to_many(
            "books",
            BookStore::id(),
            Book::store_id(),
            Book::fetcher().by(|book| book.field(Book::name()).field(Book::edition())),
        )
    });
    let result = client
        .create_query(BookStore::table())
        .where_(BookStore::id().eq(1_i64))
        .select(BookStore::table().fetch(fetcher))
        .execute_json()
        .await
        .unwrap();

    // 断言一对多 Fetcher 使用二段批量加载，并装配成子数组。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 1,
            "NAME": "O'REILLY",
            "books": [
                {
                    "NAME": "GraphQL in Action",
                    "EDITION": 1
                },
                {
                    "NAME": "Learning Rust",
                    "EDITION": 1
                }
            ]
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_one_to_many_fetcher_options() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    seed_book(client.pool()).await;
    seed_second_book(client.pool()).await;
    seed_third_book(client.pool()).await;

    let fetcher = BookStore::fetcher().by(|store| {
        store.field(BookStore::name()).one_to_many_with_options(
            "books",
            BookStore::id(),
            Book::store_id(),
            Book::fetcher().by(|book| book.field(Book::name()).field(Book::edition())),
            CollectionFetchOptions::new()
                .filter(Book::edition().eq(1_i32))
                .order_by(Book::name().desc())
                .limit(1),
        )
    });
    let result = client
        .create_query(BookStore::table())
        .where_(BookStore::id().eq(1_i64))
        .select(BookStore::table().fetch(fetcher))
        .execute_json()
        .await
        .unwrap();

    // 断言集合 Fetcher 的过滤、排序和 limit 会按父对象独立生效。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 1,
            "NAME": "O'REILLY",
            "books": [
                {
                    "NAME": "Learning Rust",
                    "EDITION": 1
                }
            ]
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_one_to_many_exists_query() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    seed_book(client.pool()).await;

    let books = BookStore::id().one_to_many(Book::store_id());
    let result = client
        .create_query(BookStore::table())
        .where_(books.exists(Book::name().eq("GraphQL in Action")))
        .select(
            BookStore::table()
                .fetch(BookStore::fetcher().by(|store| store.field(BookStore::name()))),
        )
        .execute_json()
        .await
        .unwrap();

    // 断言一对多关联谓词可以作为隐式 EXISTS 子查询真实执行。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 1,
            "NAME": "O'REILLY"
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_many_to_many_fetcher_as_nested_json_array() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    seed_book(client.pool()).await;
    seed_authors(client.pool()).await;
    seed_book_author_mapping(client.pool()).await;

    let fetcher = Book::fetcher().by(|book| {
        book.field(Book::name()).many_to_many(
            "authors",
            Book::id(),
            ManyToManyJoin {
                table_name: "BOOK_AUTHOR_MAPPING",
                source_column: "BOOK_ID",
                target_column: "AUTHOR_ID",
                target_field: Author::id(),
            },
            Author::fetcher().by(|author| {
                author
                    .field(Author::first_name())
                    .field(Author::last_name())
            }),
        )
    });
    let result = client
        .create_query(Book::table())
        .where_(Book::id().eq(10_i64))
        .select(Book::table().fetch(fetcher))
        .execute_json()
        .await
        .unwrap();

    // 断言多对多 Fetcher 通过中间表批量加载，并装配成子数组。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 10,
            "NAME": "GraphQL in Action",
            "authors": [
                {
                    "FIRST_NAME": "Alex",
                    "LAST_NAME": "Banks"
                },
                {
                    "FIRST_NAME": "Eve",
                    "LAST_NAME": "Porcello"
                }
            ]
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_many_to_many_fetcher_options() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    seed_book(client.pool()).await;
    seed_authors(client.pool()).await;
    seed_book_author_mapping(client.pool()).await;

    let fetcher = Book::fetcher().by(|book| {
        book.field(Book::name()).many_to_many_with_options(
            "authors",
            Book::id(),
            ManyToManyJoin {
                table_name: "BOOK_AUTHOR_MAPPING",
                source_column: "BOOK_ID",
                target_column: "AUTHOR_ID",
                target_field: Author::id(),
            },
            Author::fetcher().by(|author| {
                author
                    .field(Author::first_name())
                    .field(Author::last_name())
            }),
            CollectionFetchOptions::new()
                .order_by(Author::last_name().desc())
                .limit(1),
        )
    });
    let result = client
        .create_query(Book::table())
        .where_(Book::id().eq(10_i64))
        .select(Book::table().fetch(fetcher))
        .execute_json()
        .await
        .unwrap();

    // 断言多对多集合 Fetcher 的排序和 limit 会经中间表批量加载生效。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 10,
            "NAME": "GraphQL in Action",
            "authors": [
                {
                    "FIRST_NAME": "Eve",
                    "LAST_NAME": "Porcello"
                }
            ]
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_many_to_many_exists_query() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    seed_book(client.pool()).await;
    seed_authors(client.pool()).await;
    seed_book_author_mapping(client.pool()).await;

    let authors = Book::id().many_to_many(ManyToManyJoin {
        table_name: "BOOK_AUTHOR_MAPPING",
        source_column: "BOOK_ID",
        target_column: "AUTHOR_ID",
        target_field: Author::id(),
    });
    let result = client
        .create_query(Book::table())
        .where_(authors.exists(Author::last_name().eq("Porcello")))
        .select(Book::table().fetch(Book::fetcher().by(|book| book.field(Book::name()))))
        .execute_json()
        .await
        .unwrap();

    // 断言多对多关联谓词可以通过中间表生成隐式 EXISTS 子查询。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 10,
            "NAME": "GraphQL in Action"
        })]
    );
}

#[tokio::test]
async fn sqlx_client_should_execute_save_command() {
    let client = sqlite_client().await;
    seed_book_store(client.pool()).await;
    let draft = BookStore::draft(|store| {
        store
            .set(BookStore::id(), 1_i64)
            .set(BookStore::name(), "O'REILLY+")
            .set_null(BookStore::website())
    });

    let execution = client
        .save(draft)
        .set_mode(rimmer::save::SaveMode::UpdateOnly)
        .execute()
        .await
        .unwrap();

    // 断言保存命令真正影响了数据库行。
    assert_eq!(execution.rows_affected, 1);

    let result = JimmerClient::new()
        .create_query(BookStore::table())
        .where_(BookStore::id().eq(1_i64))
        .select(
            BookStore::table().fetch(
                BookStore::fetcher()
                    .by(|store| store.field(BookStore::name()).field(BookStore::website())),
            ),
        )
        .build()
        .unwrap()
        .execute_json(client.pool())
        .await
        .unwrap();

    // 断言显式 null 经真实数据库执行后被保存下来。
    assert_eq!(
        result.rows,
        vec![json!({
            "ID": 1,
            "NAME": "O'REILLY+",
            "WEBSITE": null
        })]
    );
}

#[tokio::test]
async fn save_plan_should_execute_without_client_wrapper() {
    let client = sqlite_client().await;
    let draft = BookStore::draft(|store| {
        store
            .set(BookStore::id(), 2_i64)
            .set(BookStore::name(), "MANNING")
            .set(BookStore::website(), "https://www.manning.com")
    });
    let plan = JimmerClient::new()
        .save(draft)
        .set_mode(rimmer::save::SaveMode::InsertOnly)
        .build()
        .unwrap();

    let execution = plan.execute(client.pool()).await.unwrap();

    // 断言 plan 层也可以直接执行，方便 CLI 或迁移工具复用。
    assert_eq!(execution.rows_affected, 1);
}

async fn sqlite_client() -> SqlxJimmerClient {
    sqlx::any::install_default_drivers();
    let pool = AnyPoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .unwrap();
    sqlx::query(
        r#"
        CREATE TABLE "BOOK_STORE" (
            "ID" INTEGER PRIMARY KEY,
            "NAME" TEXT NOT NULL,
            "WEBSITE" TEXT
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        CREATE TABLE "BOOK" (
            "ID" INTEGER PRIMARY KEY,
            "NAME" TEXT NOT NULL,
            "EDITION" INTEGER NOT NULL,
            "STORE_ID" INTEGER,
            FOREIGN KEY ("STORE_ID") REFERENCES "BOOK_STORE"("ID")
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        CREATE TABLE "AUTHOR" (
            "ID" INTEGER PRIMARY KEY,
            "FIRST_NAME" TEXT NOT NULL,
            "LAST_NAME" TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        r#"
        CREATE TABLE "BOOK_AUTHOR_MAPPING" (
            "BOOK_ID" INTEGER NOT NULL,
            "AUTHOR_ID" INTEGER NOT NULL,
            FOREIGN KEY ("BOOK_ID") REFERENCES "BOOK"("ID"),
            FOREIGN KEY ("AUTHOR_ID") REFERENCES "AUTHOR"("ID")
        )
        "#,
    )
    .execute(&pool)
    .await
    .unwrap();
    SqlxJimmerClient::from_pool(pool)
}

async fn seed_book_store(pool: &sqlx::AnyPool) {
    let draft = BookStore::draft(|store| {
        store
            .set(BookStore::id(), 1_i64)
            .set(BookStore::name(), "O'REILLY")
            .set(BookStore::website(), "https://www.oreilly.com")
    });
    JimmerClient::new()
        .save(draft)
        .set_mode(rimmer::save::SaveMode::InsertOnly)
        .build()
        .unwrap()
        .execute(pool)
        .await
        .unwrap();
}

async fn seed_book(pool: &sqlx::AnyPool) {
    insert_book(pool, 10, "GraphQL in Action", 1, 1).await;
}

async fn seed_second_book(pool: &sqlx::AnyPool) {
    insert_book(pool, 11, "Learning Rust", 1, 1).await;
}

async fn seed_third_book(pool: &sqlx::AnyPool) {
    insert_book(pool, 12, "Rust Deep Dive", 2, 1).await;
}

async fn seed_authors(pool: &sqlx::AnyPool) {
    insert_author(pool, 100, "Alex", "Banks").await;
    insert_author(pool, 101, "Eve", "Porcello").await;
}

async fn seed_book_author_mapping(pool: &sqlx::AnyPool) {
    insert_book_author_mapping(pool, 10, 100).await;
    insert_book_author_mapping(pool, 10, 101).await;
}

async fn insert_book(pool: &sqlx::AnyPool, id: i64, name: &str, edition: i32, store_id: i64) {
    let draft = Book::draft(|book| {
        book.set(Book::id(), id)
            .set(Book::name(), name)
            .set(Book::edition(), edition)
            .set(Book::store_id(), store_id)
    });
    JimmerClient::new()
        .save(draft)
        .set_mode(rimmer::save::SaveMode::InsertOnly)
        .build()
        .unwrap()
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_author(pool: &sqlx::AnyPool, id: i64, first_name: &str, last_name: &str) {
    let draft = Author::draft(|author| {
        author
            .set(Author::id(), id)
            .set(Author::first_name(), first_name)
            .set(Author::last_name(), last_name)
    });
    JimmerClient::new()
        .save(draft)
        .set_mode(rimmer::save::SaveMode::InsertOnly)
        .build()
        .unwrap()
        .execute(pool)
        .await
        .unwrap();
}

async fn insert_book_author_mapping(pool: &sqlx::AnyPool, book_id: i64, author_id: i64) {
    sqlx::query(
        r#"
        INSERT INTO "BOOK_AUTHOR_MAPPING" ("BOOK_ID", "AUTHOR_ID")
        VALUES (?, ?)
        "#,
    )
    .bind(book_id)
    .bind(author_id)
    .execute(pool)
    .await
    .unwrap();
}

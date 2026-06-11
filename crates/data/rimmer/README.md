# rimmer

`rimmer` 是一个 Jimmer 风格的 Rust ORM 原型，目标是先复刻核心用法体验：

- 用实体元模型生成强类型字段。
- 用 `JimmerClient::create_query(...).where_(...).select(...).build()` 表达查询。
- 用 `new_fetcher(...).by(...)` 表达返回对象形状，并可导出为 JSON。
- 用集合关联路径生成 `EXISTS` 隐式子查询。
- 用 `CollectionFetchOptions` 配置集合 Fetcher 的过滤、排序和每父对象分页。
- 用 `new_draft(...).by(...)` 区分“字段未指定”和“字段显式为 null”。

当前版本支持生成 SQL plan，也提供基于 `sqlx::AnyPool` 的执行器。Fetcher 不设计独立 DTO 语言，默认用 JSON 这种通用格式承载对象形状；后续如果需要二进制协议，可以在同一份 `FetchShape` 语义上补 protobuf schema。

推荐用 derive 生成 Jimmer 风格元模型：

```rust
use rimmer::{JimmerClient, QueryBuilderExt};

#[derive(rimmer::Entity)]
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
}

let simple_book = Book::fetcher().by(|book| {
    book.field(Book::name())
        .field(Book::edition())
});

let plan = JimmerClient::new()
    .create_query(Book::table())
    .where_(Book::name().eq_if_not_blank(Some("GraphQL in Action")))
    .select(Book::table().fetch(simple_book))
    .build()
    .unwrap();

assert!(plan.sql.contains(r#"FROM "BOOK""#));
```

也可以用 `entity!` 在没有过程宏需求时声明实体：

```rust
use rimmer::{JimmerClient, QueryBuilderExt, entity, new_fetcher};

entity! {
    pub mod book {
        pub struct Book => "BOOK" {
            id id: i64 => "ID",
            key name: String => "NAME",
            scalar edition: i32 => "EDITION",
            scalar price: f64 => "PRICE",
        }
    }
}

let simple_book = new_fetcher(book::entity()).by(|f| {
    f.field(book::name())
        .field(book::edition())
});

let plan = JimmerClient::new()
    .create_query(book::table())
    .where_(book::name().eq_if_not_blank(Some("GraphQL in Action")))
    .select(book::table().fetch(simple_book))
    .build()
    .unwrap();

assert!(plan.sql.contains(r#"FROM "BOOK""#));
```

Fetcher 形状可以直接作为 JSON 保存、传输或缓存：

```rust
# use rimmer::{JimmerClient, QueryBuilderExt, entity, new_fetcher};
# entity! {
#     pub mod book {
#         pub struct Book => "BOOK" {
#             id id: i64 => "ID",
#             key name: String => "NAME",
#             scalar edition: i32 => "EDITION",
#         }
#     }
# }
let simple_book = new_fetcher(book::entity()).by(|f| {
    f.field(book::name())
        .field(book::edition())
});

let json = simple_book.to_json().unwrap();
let restored = rimmer::Fetcher::from_json(book::entity(), &json).unwrap();
let value = simple_book.to_json_value().unwrap();
let restored_from_value = rimmer::Fetcher::from_json_value(book::entity(), value).unwrap();

assert_eq!(restored.shape(), simple_book.shape());
assert_eq!(restored_from_value.shape(), simple_book.shape());
```

接入 `sqlx` 后，可以像 Jimmer 的 `sqlClient` 一样把查询链直接执行成 JSON：

```rust
# use rimmer::{QueryBuilderExt, SqlxJimmerClient, entity, new_fetcher};
# entity! {
#     pub mod book_store {
#         pub struct BookStore => "BOOK_STORE" {
#             id id: i64 => "ID",
#             key name: String => "NAME",
#             scalar website: Option<String> => "WEBSITE",
#         }
#     }
# }
# async fn demo(sql: SqlxJimmerClient) -> rimmer::OrmResult<()> {
let stores = sql
    .create_query(book_store::table())
    .where_(book_store::name().eq_if_not_blank(Some("O'REILLY")))
    .select(book_store::table().fetch(
        new_fetcher(book_store::entity()).by(|store| {
            store
                .field(book_store::name())
                .field(book_store::website())
        })
    ))
    .execute_json()
    .await?;

assert_eq!(stores.rows.len(), 1);
# Ok(())
# }
```

`rimmer` 内部 SQL plan 统一使用 `?` 占位符；`SqlxJimmerClient::connect(...)` 会根据数据库 URL 选择方言，SQLite 保持 `?`，PostgreSQL 会在执行前渲染为 `$1`、`$2`。

Fetcher 可以声明 `many_to_one`，查询时会自动 `LEFT JOIN` 并返回嵌套 JSON：

```rust
# use rimmer::{JimmerClient, QueryBuilderExt};
#[derive(rimmer::Entity)]
#[rimmer(table = "BOOK_STORE")]
pub struct BookStore {
    #[rimmer(id, column = "ID")]
    pub id: i64,
    #[rimmer(key, column = "NAME")]
    pub name: String,
}

#[derive(rimmer::Entity)]
#[rimmer(table = "BOOK")]
pub struct Book {
    #[rimmer(id, column = "ID")]
    pub id: i64,
    #[rimmer(key, column = "NAME")]
    pub name: String,
    #[rimmer(many_to_one, column = "STORE_ID")]
    pub store_id: Option<i64>,
}

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
    .select(Book::table().fetch(fetcher))
    .build()
    .unwrap();

assert!(plan.sql.contains("LEFT JOIN"));
```

`one_to_many` 使用二段批量加载，不会把父对象用 join 展平成重复行：

```rust
# use rimmer::{JimmerClient, QueryBuilderExt};
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK_STORE")]
# pub struct BookStore {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
# }
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK")]
# pub struct Book {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
#     #[rimmer(many_to_one, column = "STORE_ID")]
#     pub store_id: Option<i64>,
# }
let fetcher = BookStore::fetcher().by(|store| {
    store.field(BookStore::name()).one_to_many(
        "books",
        BookStore::id(),
        Book::store_id(),
        Book::fetcher().by(|book| book.field(Book::name())),
    )
});

let plan = JimmerClient::new()
    .create_query(BookStore::table())
    .select(BookStore::table().fetch(fetcher))
    .build()
    .unwrap();

assert!(plan.sql.contains(r#"FROM "BOOK_STORE""#));
```

集合 Fetcher 可以配置关联级过滤、排序和分页；`limit`/`offset` 按每个父对象独立生效：

```rust
# use rimmer::{CollectionFetchOptions, JimmerClient, QueryBuilderExt};
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK_STORE")]
# pub struct BookStore {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
# }
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK")]
# pub struct Book {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
#     #[rimmer(key, column = "EDITION")]
#     pub edition: i32,
#     #[rimmer(many_to_one, column = "STORE_ID")]
#     pub store_id: Option<i64>,
# }
let fetcher = BookStore::fetcher().by(|store| {
    store.field(BookStore::name()).one_to_many_with_options(
        "books",
        BookStore::id(),
        Book::store_id(),
        Book::fetcher().by(|book| {
            book.field(Book::name())
                .field(Book::edition())
        }),
        CollectionFetchOptions::new()
            .filter(Book::edition().eq(1_i32))
            .order_by(Book::name().desc())
            .limit(1),
    )
});

let plan = JimmerClient::new()
    .create_query(BookStore::table())
    .select(BookStore::table().fetch(fetcher))
    .build()
    .unwrap();

assert!(plan.sql.contains(r#"FROM "BOOK_STORE""#));
```

同一套集合关联路径可以生成 Jimmer 风格的隐式子查询谓词：

```rust
# use rimmer::{JimmerClient, QueryBuilderExt};
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK_STORE")]
# pub struct BookStore {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
# }
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK")]
# pub struct Book {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
#     #[rimmer(key, column = "EDITION")]
#     pub edition: i32,
#     #[rimmer(many_to_one, column = "STORE_ID")]
#     pub store_id: Option<i64>,
# }
let books = BookStore::id().one_to_many(Book::store_id());
let plan = JimmerClient::new()
    .create_query(BookStore::table())
    .where_(books.exists(
        Book::name()
            .eq("GraphQL in Action")
            .and(Book::edition().eq(1_i32)),
    ))
    .select(
        BookStore::table().fetch(
            BookStore::fetcher().by(|store| store.field(BookStore::name())),
        ),
    )
    .build()
    .unwrap();

assert!(plan.sql.contains("EXISTS"));
```

`many_to_many` 同样使用中间表批量加载，Fetcher 形状仍然可以直接序列化成 JSON：

```rust
# use rimmer::{JimmerClient, ManyToManyJoin, QueryBuilderExt};
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK")]
# pub struct Book {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
# }
# #[derive(rimmer::Entity)]
# #[rimmer(table = "AUTHOR")]
# pub struct Author {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "FIRST_NAME")]
#     pub first_name: String,
#     #[rimmer(key, column = "LAST_NAME")]
#     pub last_name: String,
# }
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

let json = fetcher.to_pretty_json().unwrap();
assert!(json.contains(r#""joinTable""#));

let plan = JimmerClient::new()
    .create_query(Book::table())
    .select(Book::table().fetch(fetcher))
    .build()
    .unwrap();

assert!(plan.sql.contains(r#"FROM "BOOK""#));
assert!(!plan.sql.contains("BOOK_AUTHOR_MAPPING"));
```

多对多 Fetcher 也支持同样的集合关联级配置：

```rust
# use rimmer::{CollectionFetchOptions, JimmerClient, ManyToManyJoin, QueryBuilderExt};
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK")]
# pub struct Book {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
# }
# #[derive(rimmer::Entity)]
# #[rimmer(table = "AUTHOR")]
# pub struct Author {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "FIRST_NAME")]
#     pub first_name: String,
#     #[rimmer(key, column = "LAST_NAME")]
#     pub last_name: String,
# }
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

let json = fetcher.to_json().unwrap();
assert!(json.contains(r#""collectionOptions""#));
```

多对多关联路径也可以生成隐式子查询：

```rust
# use rimmer::{JimmerClient, ManyToManyJoin, QueryBuilderExt};
# #[derive(rimmer::Entity)]
# #[rimmer(table = "BOOK")]
# pub struct Book {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "NAME")]
#     pub name: String,
# }
# #[derive(rimmer::Entity)]
# #[rimmer(table = "AUTHOR")]
# pub struct Author {
#     #[rimmer(id, column = "ID")]
#     pub id: i64,
#     #[rimmer(key, column = "LAST_NAME")]
#     pub last_name: String,
# }
let authors = Book::id().many_to_many(ManyToManyJoin {
    table_name: "BOOK_AUTHOR_MAPPING",
    source_column: "BOOK_ID",
    target_column: "AUTHOR_ID",
    target_field: Author::id(),
});
let plan = JimmerClient::new()
    .create_query(Book::table())
    .where_(authors.exists(Author::last_name().eq("Porcello")))
    .select(
        Book::table().fetch(
            Book::fetcher().by(|book| book.field(Book::name())),
        ),
    )
    .build()
    .unwrap();

assert!(plan.sql.contains("BOOK_AUTHOR_MAPPING"));
```

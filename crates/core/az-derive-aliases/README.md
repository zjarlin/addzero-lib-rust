# az-derive-aliases

为重复的 derive 组合提供可复用的宏别名，减少样板代码，让结构体/enum 定义保持简洁。

## 功能

- **deserialize_debug** — 带 `Debug` + `Deserialize` 的只读响应/输入类型
- **deserialize_clone_debug** — 带 `Clone` + `Debug` + `Deserialize` 的只读响应/输入类型
- **serialize_debug** — 带 `Debug` + `Serialize` 的只写请求/输出类型
- **serialize_clone_debug** — 带 `Clone` + `Debug` + `Serialize` 的只写请求/输出类型
- **serialize_eq** — 带 `Serialize` + 相等/调试 trait 的只写请求/输出类型
- **serialize_copy_eq** — 在 `serialize_eq` 基础上改为 `Copy` 小类型，不增加 `Deserialize`
- **serialize_partial_eq** — 用于不能 `Eq` 的只写请求/输出类型
- **deserialize_eq** — 带 `Deserialize` + 相等/调试 trait 的只读响应/输入类型
- **deserialize_partial_eq** — 用于不能 `Eq` 的只读响应/输入类型
- **from_copy_eq_display** — 在 `from_copy_eq` 基础上增加 `Display`
- **serde_eq_no_debug** — 带 serde + 相等 trait 但保留自定义 `Debug` 的类型
- **serde_eq_redacted** — 带 serde + 相等 trait 的 redacted `Debug` 类型，适合 `derive_more::Debug` + `#[debug(skip)]`
- **error** — 带 `thiserror` + `Debug` 的基础库错误类型
- **error_eq** — 带 `thiserror` + 常用相等/调试 trait 的错误类型
- **impl_from_match!** — 用显式 pattern 映射快速生成 `From` 实现
- **from_eq** — 带 `derive_more::From` + `PartialEq` 的轻量转换枚举
- **from_display** — 带 `derive_more::From` + `Display` 的轻量值枚举
- **serde_eq** — 带 serde + 相等/调试 trait 的数据类型
- **serde_eq_hash** — 在 `serde_eq` 基础上增加 `Hash`
- **serde_eq_hash_display** — 在 `serde_eq_hash` 基础上增加 `Display`
- **serde_eq_hash_ord** — 在 `serde_eq_hash` 基础上增加 `Ord`/`PartialOrd`
- **serde_eq_hash_ord_display** — 在 `serde_eq_hash_ord` 基础上增加 `Display`
- **sqlx_from_row** — SQLx 查询结果行映射的 `FromRow`
- **shaku_component** — Shaku DI 组件类型的 `Component`
- **from_plain_eq** — 单字段包装枚举/值对象常用的 `From` + `Clone` + `Debug` + `PartialEq` + `Eq`
- **from_copy_eq** — 单字段 `Copy` 包装类型常用的 `From` + `Clone` + `Copy` + `Debug` + `PartialEq` + `Eq`
- **serde_eq_copy** — 在 `serde_eq` 基础上增加 `Copy`
- **serde_eq_copy_display** — 在 `serde_eq_copy` 基础上增加 `Display`
- **serde_eq_default_copy** — 在 `serde_eq_copy` 基础上增加 `Default` 和 `Hash`
- **serde_eq_default_copy_ord** — 在 `serde_eq_default_copy` 基础上增加 `Ord`/`PartialOrd`
- **serde_eq_default** — 在 `serde_eq` 基础上增加 `Default`
- **serde_partial_eq** — 用于包含 `f32`/`f64` 或动态 JSON 等不能 `Eq` 的 serde 数据类型
- **serde_partial_eq_default** — 在 `serde_partial_eq` 基础上增加 `Default`
- **serde_partial_eq_display** — 在 `serde_partial_eq` 基础上增加 `Display`
- **serde_code** — 带 serde（snake_case）+ strum 字符串转换/`Display` + `Hash` 的代码类型
- **serde_code_enum** — 在 `serde_code` 基础上生成 `ALL` / `as_str()` / `code()` / `from_code()`
- **serde_lower_code** — 带 serde（lowercase）+ strum 字符串转换/`Display` + `Hash` 的代码类型
- **serde_lower_code_enum** — 在 `serde_lower_code` 基础上生成 `ALL` / `as_str()` / `code()` / `from_code()`
- **serde_code_display** — 带 serde（snake_case）+ strum 字符串 code + `derive_more::Display` 自定义展示名的代码类型
- **serde_code_display_enum** — 在 `serde_code_display` 基础上生成 `ALL` / `as_str()` / `code()` / `from_code()`
- **serde_kebab_code** — 带 serde（kebab-case）+ strum 字符串转换/`Display` + `Hash` 的代码类型
- **serde_kebab_code_enum** — 在 `serde_kebab_code` 基础上生成 `ALL` / `as_str()` / `code()` / `from_code()`
- **plain_code_enum** — 不带 serde 的 code 枚举，通过 `strum` 提供 `ALL` / `as_str()` / `code()` / `from_code()`
- **plain_code_display_enum** — 不带 serde 的 code/display 枚举，通过 `strum` 提供 `ALL` / `as_str()` / `code()` / `from_code()`
- **clap_code_enum** — 在 `serde_code_enum` 基础上增加 Clap `ValueEnum`
- **serde_code_default** — 在 `serde_code` 基础上增加 `Default`
- **serde_code_default_enum** — 在 `serde_code_default` 基础上生成 `ALL` / `as_str()` / `code()` / `from_code()`
- **serde_code_ord** — 在 `serde_code` 基础上增加 `Ord`/`PartialOrd`
- **serde_code_ord_enum** — 在 `serde_code_ord` 基础上生成 `ALL` / `as_str()` / `code()` / `from_code()`
- **serde_code_default_ord** — 在 `serde_code_ord` 基础上增加 `Default`
- **serde_code_default_ord_enum** — 在 `serde_code_default_ord` 基础上生成 `ALL` / `as_str()` / `code()` / `from_code()`
- **plain_eq** — 纯内存结构体/枚举的 `Clone` + `Debug` + `Eq` + `PartialEq`
- **plain_clone** — 纯内存句柄/配置壳子的 `Clone`
- **plain_copy** — 纯内存只读值对象的 `Clone` + `Copy`
- **plain_default** — 纯内存状态/缓存的 `Default`
- **plain_debug** — 纯内存结果/会话/标记类型的 `Debug`
- **plain_clone_debug** — 纯内存句柄/配置壳子的 `Clone` + `Debug`
- **plain_clone_redacted** — 纯内存句柄/客户端的 `Clone` + redacted `Debug`
- **plain_default_debug** — 纯内存状态/缓存的 `Debug` + `Default`
- **plain_default_clone** — 纯内存句柄/状态的 `Clone` + `Default`
- **plain_default_clone_debug** — 纯内存句柄/状态的 `Clone` + `Debug` + `Default`
- **plain_clone_debug_display** — 在 `plain_clone_debug` 基础上增加 `Display`
- **plain_eq_no_debug** — 在 `plain_eq` 基础上去掉生成的 `Debug`
- **plain_eq_redacted** — 带相等 trait 且使用 `derive_more::Debug` + `#[debug(skip)]` 脱敏的纯内存类型
- **plain_eq_hash** — 在 `plain_eq` 基础上增加 `Hash`
- **plain_eq_hash_display** — 在 `plain_eq_hash` 基础上增加 `Display`
- **plain_partial_eq** — 纯内存结构体/枚举的 `Clone` + `Debug` + `PartialEq`
- **plain_partial_eq_display** — 在 `plain_partial_eq` 基础上增加 `Display`
- **plain_default_eq** — 在 `plain_eq` 基础上增加 `Default`
- **plain_default_partial_eq** — 在 `plain_partial_eq` 基础上增加 `Default`
- **plain_copy_eq** — 在 `plain_eq` 基础上增加 `Copy`
- **plain_copy_eq_display** — 在 `plain_copy_eq` 基础上增加 `Display`
- **plain_copy_eq_hash** — 在 `plain_copy_eq` 基础上增加 `Hash`
- **plain_copy_eq_hash_display** — 在 `plain_copy_eq_hash` 基础上增加 `Display`
- **plain_eq_display** — 在 `plain_eq` 基础上增加 `Display`
- **plain_default_copy_eq** — 在 `plain_copy_eq` 基础上增加 `Default`
- **plain_default_copy_eq_display** — 在 `plain_default_copy_eq` 基础上增加 `Display`
- **clap_parser** — Clap `Parser` 结构体常用的 `Debug` + `Parser`
- **clap_args** — Clap `Args` 结构体常用的 `Debug` + `Args`
- **clap_args_default_clone** — 在 `clap_args` 基础上增加 `Clone` + `Default`
- **clap_subcommand** — Clap `Subcommand` 枚举常用的 `Debug` + `Subcommand`
- **clap_value_enum** — Clap `ValueEnum` 常用的 `Debug` + `Clone` + `Copy` + `PartialEq` + `Eq`
- **dioxus_props** — Dioxus `Props` 结构体常用的 `Clone` + `PartialEq`
- **seaorm_entity_model** — SeaORM 实体模型常用的 `Clone` + `Debug` + `PartialEq` + `DeriveEntityModel`
- **seaorm_entity_model_eq** — 在 `seaorm_entity_model` 基础上额外增加 `Eq`
- **seaorm_relation** — SeaORM relation 常用的 `Copy` + `Clone` + `Debug` + `EnumIter` + `DeriveRelation`

所有宏设计为配合 [`macro_rules_attribute::apply`](https://docs.rs/macro_rules_attribute) 使用，保持 `#[serde(...)]` 和 `#[strum(...)]` 等辅助属性对编译器和 IDE 可见。
`serde_code*_enum` 会级联复用对应的 `serde_code*` 基础 alias，只额外生成代码枚举常用的 `ALL`、`as_str()`、`code()` 和 `from_code()`，避免每个 enum 手写同一套样板方法。`serde_code*` 同时派生 `strum::Display`，所以需要拥有字符串 code 时可以直接调用 `to_string()`；如果展示名必须不同于 wire code，使用 `serde_code_display*` 并继续通过 `#[display(...)]` 标注展示值。`serde_kebab_code*` 是同一套语义的 kebab-case 变体，不带 serde 的场景则用 `plain_code_display_enum`。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-derive-aliases = { path = "../az-derive-aliases" }       # workspace 内部引用
# 或发布后：
# az-derive-aliases = "2026.5.10"                           # crates.io 引用
```

调用侧不需要直接依赖 `macro_rules_attribute`，可以使用本 crate 重新导出的 `apply`。
使用某个 alias 时，调用侧仍应显式声明该 alias 实际派生到的 crates，例如 `serde`、`thiserror`、`strum` 或 `derive_more`。
SeaORM 相关 alias 仍要求调用侧引入 `sea_orm::entity::prelude::*`，以便 `DeriveEntityModel`、`DeriveRelation` 和 `EnumIter` 在展开后保持可见。
Clap 相关 alias 仍要求调用侧引入 `clap` 的 derive 入口，通常通过该 crate 自身的 `clap` 依赖即可满足；`clap_code_enum` 还需要 `strum`。

## 用法

```rust
use az_derive_aliases::{
    apply, deserialize_debug, deserialize_eq, error, error_eq, plain_copy_eq_hash, plain_eq_hash,
    serde_code, serde_code_default_enum, serde_code_ord, serde_eq, serialize_debug, serialize_eq,
};

// 只需要反序列化的第三方响应类型
#[apply(deserialize_debug)]
struct ApiEnvelope {
    status: String,
}

#[apply(serialize_debug)]
struct ApiRequest {
    query: String,
}

#[apply(deserialize_eq)]
struct ReadModel {
    id: String,
}

#[apply(serialize_eq)]
struct WriteCommand {
    name: String,
}

// 带 thiserror 的错误类型
#[apply(error)]
enum TransportError {
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
}

// 带 thiserror 且需要相等比较的错误类型
#[apply(error_eq)]
#[error("invalid input: {0}")]
struct MyError(String);

// 带 serde 的数据类型
#[apply(serde_eq)]
struct User {
    name: String,
    age: u8,
}

// 带 serde + 字符串转换的枚举代码
#[apply(serde_code)]
enum Status {
    Active,
    Inactive,
    Pending,
}

assert_eq!(Status::Active.to_string(), "active");

// 带 serde + 字符串转换 + 常用 code helper 的枚举代码
#[apply(serde_code_default_enum)]
enum InstallKind {
    Brew,
    #[default]
    Custom,
}

// 纯内存的带 Hash 值对象
#[apply(plain_eq_hash)]
struct TableName(String);

// 纯内存的带 Hash 小枚举
#[apply(plain_copy_eq_hash)]
enum ProviderKind {
    Local,
    Remote,
}

// 带排序能力的代码枚举
#[apply(serde_code_ord)]
enum Priority {
    Low,
    Medium,
    High,
}
```

## 依赖的 crates

- `macro_rules_attribute` — 将宏作为 derive 属性应用到类型定义
- `serde` — serde 相关 alias 的 derive 和辅助属性
- `thiserror` — `error` / `error_eq`
- `derive_more` — `from_eq` / `from_plain_eq` / `from_copy_eq` / `from_copy_eq_display` / `from_display` / `serde_eq_copy_display` / `serde_partial_eq_display` / `serde_code_display*` / `plain_code_display_enum` / `serde_eq_hash_display` / `serde_eq_hash_ord_display` / `plain_clone_debug_display` / `plain_clone_redacted` / `plain_eq_redacted` / `serde_eq_redacted` / `plain_eq_display` / `plain_eq_hash_display` / `plain_partial_eq_display` / `plain_copy_eq_display` / `plain_copy_eq_hash_display` / `plain_default_copy_eq_display`
- `clap` — `clap_parser` / `clap_args` / `clap_subcommand` / `clap_value_enum` / `clap_code_enum`
- `strum` — `serde_code*` / `serde_code*_enum` / `serde_kebab_code*` / `serde_code_display*` / `plain_code_enum` / `plain_code_display_enum`
- `sea_orm` — `seaorm_entity_model*` / `seaorm_relation`

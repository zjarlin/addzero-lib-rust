# az-derive-aliases

为重复的 derive 组合提供可复用的宏别名，减少样板代码，让结构体/enum 定义保持简洁。

## 功能

- **deserialize_debug** — 带 `Debug` + `Deserialize` 的只读响应/输入类型
- **serialize_debug** — 带 `Debug` + `Serialize` 的只写请求/输出类型
- **serialize_eq** — 带 `Serialize` + 相等/调试 trait 的只写请求/输出类型
- **serialize_partial_eq** — 用于不能 `Eq` 的只写请求/输出类型
- **deserialize_eq** — 带 `Deserialize` + 相等/调试 trait 的只读响应/输入类型
- **deserialize_partial_eq** — 用于不能 `Eq` 的只读响应/输入类型
- **serde_eq_no_debug** — 带 serde + 相等 trait 但保留自定义 `Debug` 的类型
- **error_eq** — 带 `thiserror` + 常用相等/调试 trait 的错误类型
- **from_eq** — 带 `derive_more::From` + `PartialEq` 的轻量转换枚举
- **from_display** — 带 `derive_more::From` + `Display` 的轻量值枚举
- **serde_eq** — 带 serde + 相等/调试 trait 的数据类型
- **serde_eq_hash** — 在 `serde_eq` 基础上增加 `Hash`
- **serde_eq_hash_display** — 在 `serde_eq_hash` 基础上增加 `Display`
- **serde_eq_hash_ord** — 在 `serde_eq_hash` 基础上增加 `Ord`/`PartialOrd`
- **serde_eq_hash_ord_display** — 在 `serde_eq_hash_ord` 基础上增加 `Display`
- **serde_eq_copy** — 在 `serde_eq` 基础上增加 `Copy`
- **serde_eq_copy_display** — 在 `serde_eq_copy` 基础上增加 `Display`
- **serde_eq_default_copy** — 在 `serde_eq_copy` 基础上增加 `Default` 和 `Hash`
- **serde_eq_default_copy_ord** — 在 `serde_eq_default_copy` 基础上增加 `Ord`/`PartialOrd`
- **serde_eq_default** — 在 `serde_eq` 基础上增加 `Default`
- **serde_partial_eq** — 用于包含 `f32`/`f64` 或动态 JSON 等不能 `Eq` 的 serde 数据类型
- **serde_partial_eq_default** — 在 `serde_partial_eq` 基础上增加 `Default`
- **serde_code** — 带 serde（snake_case）+ strum 字符串转换 + `Hash` 的代码类型
- **serde_code_enum** — 在 `serde_code` 基础上生成 `ALL` / `code()` / `from_code()`
- **serde_code_default** — 在 `serde_code` 基础上增加 `Default`
- **serde_code_default_enum** — 在 `serde_code_default` 基础上生成 `ALL` / `code()` / `from_code()`
- **serde_code_ord** — 在 `serde_code` 基础上增加 `Ord`/`PartialOrd`
- **serde_code_ord_enum** — 在 `serde_code_ord` 基础上生成 `ALL` / `code()` / `from_code()`
- **serde_code_default_ord** — 在 `serde_code_ord` 基础上增加 `Default`
- **serde_code_default_ord_enum** — 在 `serde_code_default_ord` 基础上生成 `ALL` / `code()` / `from_code()`
- **plain_eq** — 纯内存结构体/枚举的 `Clone` + `Debug` + `Eq` + `PartialEq`
- **plain_clone** — 纯内存句柄/配置壳子的 `Clone`
- **plain_clone_debug** — 纯内存句柄/配置壳子的 `Clone` + `Debug`
- **plain_clone_debug_display** — 在 `plain_clone_debug` 基础上增加 `Display`
- **plain_eq_no_debug** — 在 `plain_eq` 基础上去掉生成的 `Debug`
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

所有宏设计为配合 [`macro_rules_attribute::apply`](https://docs.rs/macro_rules_attribute) 使用，保持 `#[serde(...)]` 和 `#[strum(...)]` 等辅助属性对编译器和 IDE 可见。
`serde_code*_enum` 会级联复用对应的 `serde_code*` 基础 alias，只额外生成代码枚举常用的 `ALL`、`code()` 和 `from_code()`，避免每个 enum 手写同一套样板方法。

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

## 用法

```rust
use az_derive_aliases::{
    apply, deserialize_debug, deserialize_eq, error_eq, plain_copy_eq_hash, plain_eq_hash,
    serde_code, serde_code_default_enum, serde_code_ord, serde_eq, serialize_debug, serialize_eq,
};
use thiserror::Error;

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
- `thiserror` — `error_eq`
- `derive_more` — `from_eq` / `from_display` / `serde_eq_copy_display` / `serde_eq_hash_display` / `serde_eq_hash_ord_display` / `plain_clone_debug_display` / `plain_eq_display` / `plain_eq_hash_display` / `plain_partial_eq_display` / `plain_copy_eq_display` / `plain_copy_eq_hash_display` / `plain_default_copy_eq_display`
- `strum` — `serde_code*` / `serde_code*_enum`

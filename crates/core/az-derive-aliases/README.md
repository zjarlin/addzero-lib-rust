# az-derive-aliases

为重复的 derive 组合提供可复用的宏别名，减少样板代码，让结构体/enum 定义保持简洁。

## 功能

- **error_eq** — 带 `thiserror` + 常用相等/调试 trait 的错误类型
- **serde_eq** — 带 serde + 相等/调试 trait 的数据类型
- **serde_eq_default** — 在 `serde_eq` 基础上增加 `Default`
- **serde_code** — 带 serde（snake_case）+ strum 字符串转换的代码类型
- **serde_code_default** — 在 `serde_code` 基础上增加 `Default`
- **serde_code_ord** — 在 `serde_code` 基础上增加 `Ord`/`PartialOrd`

所有宏设计为配合 [`macro_rules_attribute::apply`](https://docs.rs/macro_rules_attribute) 使用，保持 `#[serde(...)]` 和 `#[strum(...)]` 等辅助属性对编译器和 IDE 可见。

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-derive-aliases = { path = "../az-derive-aliases" }       # workspace 内部引用
# 或发布后：
# az-derive-aliases = "2026.5.10"                           # crates.io 引用
macro_rules_attribute = "0.2"
```

## 用法

```rust
use az_derive_aliases::apply;

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

// 带排序能力的代码枚举
#[apply(serde_code_ord)]
#[apply(serde_eq_default)]
enum Priority {
    Low,
    Medium,
    High,
}
```

## 依赖的 crates

- `macro_rules_attribute` — 将宏作为 derive 属性应用到类型定义
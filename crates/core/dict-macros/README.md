# az-dict-macros

数据字典枚举生成宏，从 JSON 规格文件自动生成类型安全的 Rust 枚举，消除手写枚举与字典数据不一致的风险。

## 功能

- 从 JSON 字典规格编译期生成完整枚举类型
- 自动提供 `code()`、`label()`、`description()`、`raw_value()`、`meta_json()` 等方法，并把 `Display` 直接派生到字典 label
- 支持开枚举（`open_enum`），未知值归入 `Unknown(T)` 变体
- 支持 `i64` 和 `&'static str` 两种原始值类型
- 生成的枚举显式派生 `Debug`、`Clone`、`Copy`、`PartialEq`、`Eq`、`Hash`、`Display`

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-dict-macros = { path = "../dict-macros" }   # workspace 内部引用
az-dict-spec = { path = "../dict-spec" }        # 依赖的规格解析 crate
derive_more = "2"                                  # 生成枚举 Display 派生所需
# 或发布后：
# az-dict-macros = "0.1"                            # crates.io 引用
# az-dict-spec = "0.1"
# derive_more = "2"
```

## 用法

```rust
use az_dict_macros::dict_enum;

dict_enum!(
    name = Gender,
    dict = "gender",
    spec = include_str!("gender.json")
);

// 生成的枚举提供了类型安全的字典访问：
let g = Gender::Male;
assert_eq!(g.code(), "male");
assert_eq!(g.to_string(), "男");
assert_eq!(g.raw_value(), "male");

// 遍历所有字典项
for item in Gender::items() {
    println!("{}: {}", item.code, item.label);
}
```

## 依赖的 crates

- `az-dict-spec` - 字典 JSON 规格解析与数据模型
- `proc-macro2` - 过程宏 token 流操作
- `quote` - Rust 代码生成
- `syn` - Rust 语法解析
- `serde_json` - JSON 序列化（用于 meta 字段）
- `derive_more` - 生成枚举的 `Display` 派生

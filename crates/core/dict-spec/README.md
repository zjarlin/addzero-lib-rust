# az-dict-spec

字典（枚举）数据的规范定义与校验，用于在应用层统一描述、校验和序列化字典型数据。

## 功能

- **DictionarySpec** — 字典规范结构体，定义字典的代码、名称、作用域、原始值类型和条目列表
- **DictionaryItemSpec** — 字典条目规范，包含 code、label、原始值和排序索引
- **DictEnumItem** — 编译期静态字典枚举项，适用于代码内嵌常量枚举
- **RawValueKind** — 原始值类型枚举（整数/字符串）
- **校验逻辑** — 自动校验字段非空、条目不重复、原始值类型一致性
- **JSON 序列化/反序列化** — 支持 camelCase 风格的 JSON 格式

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-dict-spec = { path = "../dict-spec" }  # workspace 内部引用
# 或发布后：
# az-dict-spec = "0.1"                       # crates.io 引用
```

## 用法

```rust
use az_dict_spec::api::{DictionarySpec, RawValueKind};

let json = r#"{
    "code": "gender",
    "name": "性别",
    "scope": "global",
    "rawValueKind": "int",
    "items": [
        { "code": "male", "label": "男", "rawIntValue": 0 },
        { "code": "female", "label": "女", "rawIntValue": 1 }
    ]
}"#;

let spec = DictionarySpec::from_json_str(json).expect("校验通过");
assert_eq!(spec.code, "gender");
assert_eq!(spec.items.len(), 2);
```

## 依赖的 crates

- `serde` — 序列化/反序列化支持
- `serde_json` — JSON 处理
- `anyhow` — 错误上下文与统一 `Result`

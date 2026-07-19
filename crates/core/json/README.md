# az-json

为 `serde_json` 扩展实用 JSON 工具函数的便捷辅助库，支持点路径查询、类型安全提取、深度合并、展平以及美化打印。

## 功能

- **`get_value`** — 使用点分隔路径（如 `"a.b.c"`）查询嵌套 JSON 值
- **`get_string` / `get_i64` / `get_f64` / `get_bool`** — 类型安全的路径值提取
- **`merge`** — 深度合并两个 JSON 对象，递归处理嵌套对象
- **`flatten`** — 将嵌套 JSON 展平为 `HashMap<String, Value>`，键为点分隔路径
- **`pretty`** — JSON 值的美化格式化输出

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-json = { path = "../json" }       # workspace 内部引用
# 或发布后：
# az-json = "0.1"                        # crates.io 引用
```

## 用法

```rust
use serde_json::json;
use az_json::api::{flatten, get_i64, get_string, get_value, merge, pretty};

// 点路径查询
let data = json!({ "user": { "name": "Alice", "age": 30 } });
let name = get_string(&data, "user.name"); // Some("Alice".to_string())
let age = get_i64(&data, "user.age");      // Some(30)

// 深度合并
let mut base = json!({ "a": 1, "b": { "x": 10 } });
let overlay = json!({ "b": { "y": 20 }, "c": 3 });
merge(&mut base, &overlay);
// base == { "a": 1, "b": { "x": 10, "y": 20 }, "c": 3 }

// 展平嵌套结构
let data = json!({ "a": { "b": 1 }, "c": [10, 20] });
let flat = flatten(&data);
// flat["a.b"] == 1, flat["c.0"] == 10

// 美化打印
let formatted = pretty(&json!({ "hello": "world" }));
```

## 依赖的 crates

- `serde` — 序列化框架
- `serde_json` — JSON 值类型与序列化支持

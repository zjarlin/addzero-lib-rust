# az-reflection

运行时类型反射与带 TTL 的线程安全缓存工具库，支持字段元信息提取、SQL 表名解析、驼峰转蛇形列名推断。

## 功能

- `MetaInfo` trait 与 `FieldInfo` 结构：在运行时获取类型字段名、描述、列名、嵌套对象及集合的递归展开
- `field_info!` / `reflect_meta!` 宏：声明式为结构体实现元信息描述，避免手写样板代码
- `ExpiringCache<K, V>`：基于 `Mutex` 的线程安全 KV 缓存，支持单条 TTL 过期与最大容量自动淘汰
- `extract_table_name`：从 SQL 语句中提取 `FROM` 子句的表名
- `guess_column_name`：将驼峰字段名自动转换为蛇形数据库列名
- `is_new` / `is_not_new`：检测对象是否为"空白"状态（所有字段均为 null/空字符串/空数组）

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-reflection = { path = "../reflection" }       # workspace 内部引用
# 或发布后：
# az-reflection = "0.1"                              # crates.io 引用
```

## 用法

```rust
use az_reflection::cache::ExpiringCache;
use az_reflection::metainfo::{FieldInfo, MetaInfo};
use az_reflection::{field_info, reflect_meta};
use std::num::NonZeroUsize;
use std::time::Duration;

// 使用宏声明结构体的元信息
struct User {
    name: String,
    age: u32,
}

reflect_meta!(User, description = "用户", [
    field_info!(name: String => "用户名", column = "user_name"),
    field_info!(age: u32 => "年龄"),
]);

// 读取字段描述
let fields = User::field_infos();
assert_eq!(fields[0].field_name, "name");

// 使用缓存
let cache = ExpiringCache::new(Duration::from_secs(60), NonZeroUsize::new(100).unwrap());
let value = cache.compute_if_absent("key".to_owned(), |_| 42).unwrap();
assert_eq!(value, 42);
```

## 依赖的 crates

- `anyhow` — 缓存操作错误返回
- `regex` — 正则表达式，用于 SQL 表名提取
- `serde` / `serde_json` — JSON 序列化，用于值判断辅助函数

# az-stream-wrapper

对集合进行链式条件筛选的流式查询包装器，提供类似 MyBatis-Plus LambdaQueryWrapper 的链式过滤 API。

## 功能

- **等值匹配**（`eq`）：精确字符串匹配过滤
- **模糊匹配**（`like`）：大小写不敏感的子串匹配
- **集合匹配**（`in`）：判断字段值是否在候选集合内
- **逻辑组合**（`or` / `not` / `negate`）：灵活组合多个谓词条件
- **结果提取**（`list` / `one`）：收集所有匹配项或返回第一个匹配项
- **宏快捷方式**（`stream_query!`）：一行代码启动链式查询

## 安装

在 `Cargo.toml` 中添加：
```toml
[dependencies]
az-stream-wrapper = { path = "../az-stream-wrapper" }  # workspace 内部引用
# 或发布后：
# az-stream-wrapper = "0.1"                              # crates.io 引用
```

## 用法

```rust
use az_stream_wrapper::api::lambdaquery;

let items = || vec!["apple", "banana", "avocado", "blueberry"];

// 模糊匹配包含 "av" 的元素
let result = lambdaquery(items())
    .like(true, |s: &&str| *s, "av")
    .list();
assert_eq!(result, vec!["avocado"]);

// 等值 + 或条件组合
let result = lambdaquery(items())
    .eq(true, |s: &&str| *s, "apple")
    .or()
    .eq(true, |s: &&str| *s, "banana")
    .list();
assert_eq!(result, vec!["apple", "banana"]);

// 使用宏快捷方式
let result = az_stream_wrapper::stream_query!(items())
    .like(true, |s: &&str| *s, "berry")
    .one();
assert_eq!(result, Some("blueberry"));
```

## 依赖的 crates

无外部依赖，仅使用标准库。

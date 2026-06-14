# az-model

数据库实体、分页和审计的通用模型 trait 库。

## 功能

### 实体 trait
- `Identifiable` — 具有主键的实体，通过关联类型 `Id` 标识主键类型
- `Timestamped` — 追踪创建和更新时间戳的实体
- `SoftDeletable` — 支持软删除的实体（通过 `deleted_at` 时间戳）
- `Auditable` — 追踪创建者和更新者的实体（扩展 `Timestamped`）

### 分页支持
- `Pageable` — 分页请求参数 trait，自动计算 offset
- `PageResult<T>` — 泛型分页响应容器，支持总页数计算、前后页判断等便捷方法

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-model = { path = "../az-model" }       # workspace 内部引用
# 或发布后：
# az-model = "0.1"                        # crates.io 引用
```

## 用法

```rust
use az_model::api::{Identifiable, Timestamped, SoftDeletable, Auditable, Pageable, PageResult};
use chrono::{DateTime, Utc};

// 实现 trait
struct User {
    id: u64,
    name: String,
    created_at: Option<DateTime<Utc>>,
    updated_at: Option<DateTime<Utc>>,
    deleted_at: Option<DateTime<Utc>>,
    created_by: Option<String>,
    updated_by: Option<String>,
}

impl Identifiable for User {
    type Id = u64;
    fn id(&self) -> &u64 { &self.id }
}

impl Timestamped for User {
    fn created_at(&self) -> Option<DateTime<Utc>> { self.created_at }
    fn updated_at(&self) -> Option<DateTime<Utc>> { self.updated_at }
}

impl SoftDeletable for User {
    fn deleted_at(&self) -> Option<DateTime<Utc>> { self.deleted_at }
}

// 分页
let page = PageResult::new(vec![1, 2], 100, 1, 10);
println!("总页数: {}", page.total_pages());
```

## 依赖的 crates

- `chrono` — 日期时间处理
- `serde` — 序列化/反序列化
- `serde_json` — JSON 支持

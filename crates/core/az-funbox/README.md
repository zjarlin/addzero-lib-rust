# az-funbox

REST API 函数元数据描述与注册中心，用于集中管理端点的路径、方法类型、参数和返回值信息。

## 功能

- **`FieldDto`** — 描述单个字段的元数据（字段名、英文名、类型、长度等），支持 serde 序列化
- **`FunBox`** — 描述一个 REST 端点的完整信息：URL、HTTP 方法、函数名、参数列表和返回值
- **`FunBoxRegistry`** — 函数注册表，支持按名称、URL 或 HTTP 方法类型检索
- **Builder 模式** — `FieldDto` 和 `FunBox` 均提供流式 builder，便于构造实例
- **serde 支持** — 所有核心类型均可从 JSON 等格式序列化/反序列化

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-funbox = { path = "../az-funbox" }       # workspace 内部引用
# 或发布后：
# az-funbox = "0.1"                          # crates.io 引用
```

## 用法

```rust
use az_funbox::{FunBox, FunBoxRegistry, FieldDto};

// 构造一个端点描述
let endpoint = FunBox::builder()
    .rest_url("/api/users")
    .method_type("GET")
    .fun_name("list_users")
    .parameter(FieldDto::string_field("查询关键字", "keyword"))
    .build();

// 注册到注册表
let mut registry = FunBoxRegistry::new();
registry.register(endpoint);

// 按名称检索
let found = registry.find_by_fun_name("list_users");
assert!(found.is_some());

// 获取所有注册的端点
let all = registry.all();
```

## 依赖的 crates

- `serde` — 序列化/反序列化支持

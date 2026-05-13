# az-lowcode

基于 CSS Grid 的低代码页面设计器，提供从数据模型、画布编辑、事件系统、脚本引擎到 Axum HTTP 路由的完整能力，数据持久化基于 PostgreSQL。

## 功能

- **三层数据模型**：`LayoutSchema`（布局）→ `ComponentNode`（组件节点）→ `EventBindingRecord`（事件绑定），所有类型支持 serde JSON / PG JSONB 双向序列化
- **CSS Grid 编译引擎**：将布局树转化为完整 CSS 字符串，支持断点响应式媒体查询
- **无状态画布编辑器**：提供组件放置、属性更新、删除、移动、重挂载等操作，内置网格碰撞检测
- **组件类型注册表**：内置 button、input、text、container、table、form、image、divider、az-edge 共 9 种组件，支持 JSON Schema 属性校验与运行时渲染
- **az-edge 执行卡片**：支持 curl、python、rhai、ts 四种运行变体，可声明输入输出参数，模板支持 `{{param}}` 占位符，并可生成完整 REST 接口契约
- **事件系统**：7 种内置 handler（noop / navigate / show_message / set_state / emit_event / http_call / rhai_script），支持自定义扩展，dispatch 带超时保护
- **Rhai 脚本引擎**：嵌入式沙箱执行，含操作数限制、字符串/数组大小限制、语法验证
- **PostgreSQL CRUD**：`LayoutRepository` trait 定义布局读写接口，`PgLayoutRepo` 提供 PG 实现骨架
- **Axum HTTP 路由**：覆盖布局 CRUD、节点操作、事件分发、脚本校验、组件注册等全部 REST API

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
az-lowcode = { path = "../az-lowcode" }  # workspace 内部引用
# 或发布后：
# az-lowcode = "0.1"                     # crates.io 引用
```

## 用法

```rust
use az_lowcode::{
    GridEngine, LayoutEditor, ComponentRegistry,
    ScriptEngine, LowcodeState, lowcode_router,
    LayoutSchema, GridDefinition, GridArea, ComponentNode,
};

// 1. 构造布局
let mut layout = LayoutSchema {
    grid: GridDefinition {
        columns: 12,
        rows: 8,
        gap: Some("16px".into()),
        row_height: Some("80px".into()),
        breakpoints: vec![],
    },
    children: vec![],
};

// 2. 放置组件
let btn_id = LayoutEditor::place_component(
    &mut layout,
    "root",
    "button",
    GridArea { col_start: 1, col_end: 3, row_start: 1, row_end: 2 },
    serde_json::json!({ "label": "点击我" }),
).expect("放置失败");

// 3. 编译 CSS
let css = GridEngine::compile_css(&layout);
println!("{}", css);

// 4. 组件注册表与渲染
let registry = ComponentRegistry::with_builtins();
let node = ComponentNode {
    id: "n1".into(),
    type_key: "button".into(),
    props: serde_json::json!({ "label": "OK" }),
    grid_area: GridArea { col_start: 1, col_end: 2, row_start: 1, row_end: 2 },
    children: vec![],
};
let html = registry.render(&node).expect("渲染失败");

// 5. 脚本引擎
let engine = ScriptEngine::new();
engine.validate("let x = 40 + 2").expect("语法错误");
```

## 依赖的 crates

- `axum` — HTTP 路由与请求处理
- `serde` / `serde_json` — 数据序列化（含 PG JSONB）
- `tokio` — 异步运行时（事件 dispatch 超时）
- `sqlx` — PostgreSQL 异步数据库驱动
- `uuid` — 布局 / 节点 ID 生成
- `thiserror` — 错误类型派生
- `async-trait` — 异步 trait 支持（`LayoutRepository`、`EventHandler`）
- `reqwest` — HTTP 调用 handler 发起外部请求
- `rhai` — 嵌入式脚本引擎

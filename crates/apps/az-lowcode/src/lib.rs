//! 低代码页面设计器——基于 CSS Grid 的可视化布局编辑平台。
//!
//! 本 crate 提供从数据模型、画布编辑、事件系统、脚本引擎到 Axum HTTP 路由的
//! 完整低代码页面设计能力，数据持久化基于 PostgreSQL。
//!
//! ## 核心模块
//!
//! | 模块        | 职责 |
//! |-------------|------|
//! | `schema`    | 三层数据模型：`LayoutSchema` → `ComponentNode` → `EventBindingRecord` |
//! | `grid`      | CSS Grid 编译引擎，将布局树转化为带断点响应式的 CSS 字符串 |
//! | `editor`    | 无状态画布编辑器，提供放置、更新、删除、移动、重挂载节点等操作 |
//! | `registry`  | 组件类型注册表，内置 8 种基础组件（button/input/text/container/table/form/image/divider） |
//! | `events`    | 事件系统，7 种内置 handler（noop/navigate/show_message/set_state/emit_event/http_call/rhai_script） |
//! | `scripting` | 嵌入式 Rhai 脚本引擎，带沙箱限制与语法验证 |
//! | `repo`      | PostgreSQL 布局 CRUD（`LayoutRepository` trait + `PgLayoutRepo`） |
//! | `router`    | Axum HTTP 路由层，覆盖布局 CRUD、节点操作、事件分发、脚本校验、组件注册等全部 API |
//! | `template`  | 可复用布局模板管理（骨架，待完善） |
//! | `render`    | 渲染管线，将布局树输出为预览 HTML（骨架，待完善） |
//! | `state`     | Axum 共享状态，聚合 PG 连接池、组件注册表、脚本引擎、Handler 注册表 |

pub mod editor;
pub mod events;
pub mod grid;
pub mod registry;
pub mod render;
pub mod repo;
pub mod router;
pub mod schema;
pub mod scripting;
pub mod state;
pub mod template;

// Re-export core schema types
pub use grid::{DEFAULT_COLUMNS, GridEngine, compile_css};
pub use schema::{
    Breakpoint, ComponentDefRecord, ComponentNode, EventBindingRecord, GridArea, GridDefinition,
    HandlerType, LayoutSchema,
};

// Re-export repository trait and record
pub use repo::{LayoutRecord, LayoutRepository, PgLayoutRepo, RepoError};

// Re-export registry types
pub use registry::{ComponentEntry, ComponentInfo, ComponentRegistry, RegistryError};

// Re-export editor types
pub use editor::{EditorError, LayoutEditor};

pub use scripting::{ScriptEngine, ScriptError, ValidateResponse};

pub use router::lowcode_router;
pub use state::LowcodeState;

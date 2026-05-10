//! REST API 函数元数据描述与注册中心。
//!
//! 提供 [`FunBox`] 结构体用于描述单个 REST API 端点的元数据（路径、方法类型、
//! 参数列表、返回值），以及 [`FunBoxRegistry`] 用于集中管理和检索一组函数定义。
//!
//! # 核心类型
//!
//! - [`FieldDto`] — 单个字段的描述信息（字段名、英文名、类型、长度等）。
//! - [`FunBox`] — 一个 REST 端点的完整元数据：URL、HTTP 方法、函数名、
//!   参数列表与返回值列表。
//! - [`FunBoxRegistry`] — 函数注册表，支持按名称、URL 或方法类型检索。
//! - [`AbsFunBox`] — 无状态辅助结构，提供从注册表批量获取函数定义的便捷方法。
//!
//! # 设计特点
//!
//! - 所有结构体均支持 `serde` 序列化/反序列化，便于从 JSON 等格式加载。
//! - [`FieldDto`] 和 [`FunBox`] 均提供 builder 模式构造器，适合在测试或配置
//!   代码中流式构建实例。
//! - [`FunBoxRegistry`] 提供链式 `register` / `extend` API，方便批量注册。
//!
//! # 典型用法
//!
//! ```rust
//! use az_funbox::{FunBox, FunBoxRegistry, FieldDto};
//!
//! let endpoint = FunBox::builder()
//!     .rest_url("/api/users")
//!     .method_type("GET")
//!     .fun_name("list_users")
//!     .parameter(FieldDto::string_field("查询关键字", "keyword"))
//!     .build();
//!
//! let mut registry = FunBoxRegistry::new();
//! registry.register(endpoint);
//!
//! let found = registry.find_by_fun_name("list_users");
//! assert!(found.is_some());
//! ```

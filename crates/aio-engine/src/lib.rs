//! AIO 引擎层 — 多脚本统一调度、AI 编排、任务流引擎。
//!
//! 提供：
//! - ScriptEngine trait — 统一抽象 Rhai / Python / TypeScript / Bash
//! - AiEngine trait — AI Vibe Coding 编排
//! - TaskFlow trait — 任务流编排

pub mod ai;
pub mod script;
pub mod task;

pub use ai::AiEngine;
pub use script::*;
pub use task::TaskFlow;

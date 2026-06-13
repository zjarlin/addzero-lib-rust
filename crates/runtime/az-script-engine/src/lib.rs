//! 可嵌入脚本引擎的通用契约。
//!
//! 此 crate 聚焦于一件事：定义请求/响应类型以及
//! 具体引擎（如 Rhai）可以实现的 trait。

automod::dir!(pub "src");

pub use script::{
    BoxScriptEngine, InMemoryScriptEngineRegistry, ScriptEngine, ScriptEngineFactory,
    ScriptEngineRegistry, ScriptFuture, ScriptInput, ScriptLang, ScriptOutput,
    register_engine_factory,
};

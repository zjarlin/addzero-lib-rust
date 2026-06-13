//! [`az_script_engine::ScriptEngine`] trait 的 Rhai 实现。
//!
//! 基于 [Rhai](https://rhai.rs) 脚本引擎，提供同步和异步脚本执行能力。
//! 支持变量注入、`print`/`debug` 输出捕获，以及执行结果的 JSON 序列化。
//!
//! ## 核心类型
//!
//! - [`RhaiEngine`]：实现 [`ScriptEngine`] trait，封装 Rhai 引擎实例。
//!
//! ## 安全约束
//!
//! 引擎默认设置最大操作数（100 万次）和最大模块数（32），防止脚本失控。

use az_script_engine::script::{
    BoxScriptEngine, InMemoryScriptEngineRegistry, ScriptEngine, ScriptEngineFactory,
    ScriptEngineRegistry, ScriptFuture, ScriptInput, ScriptLang, ScriptOutput,
    register_engine_factory,
};
use rhai::{Dynamic, Engine, Scope};

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

/// 基于 Rhai 的进程内脚本引擎实现。
///
/// 引擎内部持有一个可注入的 [`Engine`]，宿主可以使用 [`RhaiEngine::with_engine`] 注册
/// 自定义函数、模块或执行限制，再通过 [`ScriptEngine`] trait 统一调度。
pub struct RhaiEngine {
    engine: Mutex<Engine>,
}

/// 默认进程内 Rhai 引擎的工厂。
pub struct RhaiEngineFactory;

impl ScriptEngineFactory for RhaiEngineFactory {
    fn lang(&self) -> ScriptLang {
        ScriptLang::Rhai
    }

    fn build(&self) -> BoxScriptEngine {
        Box::new(RhaiEngine::new())
    }
}

/// 将默认 Rhai 引擎注册到已有脚本引擎注册表。
pub fn register_rhai_engine(registry: &mut dyn ScriptEngineRegistry) {
    register_engine_factory(registry, &RhaiEngineFactory);
}

/// 创建一个预装默认 Rhai 引擎的脚本引擎注册表。
#[must_use]
pub fn rhai_engine_registry() -> InMemoryScriptEngineRegistry {
    InMemoryScriptEngineRegistry::with_factories(&[&RhaiEngineFactory])
}

impl RhaiEngine {
    /// 创建带默认安全限制的 Rhai 引擎。
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(1_000_000);
        engine.set_max_modules(32);

        Self::with_engine(engine)
    }

    /// 使用预配置的 Rhai 运行时创建脚本引擎。
    ///
    /// 宿主需要注入注册函数、模块或自定义限制时使用该入口。
    pub fn with_engine(engine: Engine) -> Self {
        Self {
            engine: Mutex::new(engine),
        }
    }
}

impl Default for RhaiEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptEngine for RhaiEngine {
    fn lang(&self) -> ScriptLang {
        ScriptLang::Rhai
    }

    fn run(&self, input: ScriptInput) -> ScriptOutput {
        let start = Instant::now();
        let mut scope = Scope::new();

        for (key, value) in &input.vars {
            let dynamic: Dynamic = serde_json::from_value(value.clone()).unwrap_or_default();
            scope.push(key.clone(), dynamic);
        }

        let stdout = Arc::new(Mutex::new(String::new()));
        let stderr = Arc::new(Mutex::new(String::new()));

        let mut engine = self.engine.lock().unwrap();
        {
            let so = stdout.clone();
            let se = stderr.clone();
            engine.on_print(move |s| {
                if let Ok(mut b) = so.lock() {
                    b.push_str(s);
                }
            });
            engine.on_debug(move |s, _src, _pos| {
                if let Ok(mut b) = se.lock() {
                    b.push_str(&format!("[debug] {s}\n"));
                }
            });
        }

        let result: Result<Dynamic, _> = engine.eval_with_scope(&mut scope, &input.source);

        let mut vars = BTreeMap::new();
        for (key, _is_const, _value) in scope.iter() {
            if let Ok(json) = serde_json::to_value(scope.get_value::<Dynamic>(key)) {
                vars.insert(key.to_string(), json);
            }
        }

        let stdout_str = stdout.lock().map(|b| b.clone()).unwrap_or_default();
        let stderr_str = stderr.lock().map(|b| b.clone()).unwrap_or_default();

        match result {
            Ok(val) => {
                if let Ok(json) = serde_json::to_value(val) {
                    vars.insert("_result".to_string(), json);
                }
                ScriptOutput {
                    exit_code: 0,
                    stdout: stdout_str,
                    stderr: stderr_str,
                    vars,
                    duration_ms: start.elapsed().as_millis() as u64,
                }
            }
            Err(err) => ScriptOutput {
                exit_code: 1,
                stdout: stdout_str,
                stderr: format!("{stderr_str}\n{err}"),
                vars,
                duration_ms: start.elapsed().as_millis() as u64,
            },
        }
    }

    fn run_async<'a>(&'a self, input: ScriptInput) -> ScriptFuture<'a, ScriptOutput> {
        Box::pin(async move { self.run(input) })
    }
}

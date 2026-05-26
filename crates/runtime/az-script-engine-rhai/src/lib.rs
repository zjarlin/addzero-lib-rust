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

pub struct RhaiEngine {
    engine: Mutex<Engine>,
}

/// Factory for the default in-process Rhai engine implementation.
pub struct RhaiEngineFactory;

impl ScriptEngineFactory for RhaiEngineFactory {
    fn lang(&self) -> ScriptLang {
        ScriptLang::Rhai
    }

    fn build(&self) -> BoxScriptEngine {
        Box::new(RhaiEngine::new())
    }
}

/// Register the default Rhai engine into an existing script engine registry.
pub fn register_rhai_engine(registry: &mut dyn ScriptEngineRegistry) {
    register_engine_factory(registry, &RhaiEngineFactory);
}

/// Build a script engine registry preloaded with the default Rhai engine.
#[must_use]
pub fn rhai_engine_registry() -> InMemoryScriptEngineRegistry {
    InMemoryScriptEngineRegistry::with_factories(&[&RhaiEngineFactory])
}

impl RhaiEngine {
    pub fn new() -> Self {
        let mut engine = Engine::new();
        engine.set_max_operations(1_000_000);
        engine.set_max_modules(32);

        Self::with_engine(engine)
    }

    /// Build a script engine from a preconfigured Rhai runtime.
    ///
    /// Use this when the host wants to inject registered functions, modules, or custom limits.
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

#[cfg(test)]
mod tests {
    use super::*;
    use az_sandbox::sandbox::SandboxPolicy;

    #[test]
    fn eval_simple_expression() {
        let engine = RhaiEngine::new();
        let output = engine.run(ScriptInput {
            source: "let x = 40 + 2; x".into(),
            lang: ScriptLang::Rhai,
            vars: BTreeMap::new(),
            policy: SandboxPolicy::permissive(),
            timeout_secs: 0,
        });
        // Successful evaluation must surface a zero exit code before result decoding matters.
        assert_eq!(output.exit_code, 0);
        // The engine contract stores expression results in `_result` for downstream callers.
        assert_eq!(
            output.vars.get("_result").and_then(|v| v.as_i64()),
            Some(42)
        );
    }

    #[test]
    fn eval_with_variables() {
        let engine = RhaiEngine::new();
        let mut vars = BTreeMap::new();
        vars.insert("name".into(), serde_json::json!("AIO"));

        let output = engine.run(ScriptInput {
            source: r#""Hello, " + name + "!""#.into(),
            lang: ScriptLang::Rhai,
            vars,
            policy: SandboxPolicy::permissive(),
            timeout_secs: 0,
        });
        // Variable injection is part of the public script-engine contract, not a Rhai-only detail.
        assert_eq!(output.exit_code, 0);
        // String interpolation should round-trip through serde_json without losing the computed value.
        assert_eq!(
            output.vars.get("_result").and_then(|v| v.as_str()),
            Some("Hello, AIO!")
        );
    }

    #[test]
    fn eval_with_print() {
        let engine = RhaiEngine::new();
        let output = engine.run(ScriptInput {
            source: r#"print("hello from rhai"); 1"#.into(),
            lang: ScriptLang::Rhai,
            vars: BTreeMap::new(),
            policy: SandboxPolicy::permissive(),
            timeout_secs: 0,
        });
        // `print` output must be captured so hosts can expose script logs to callers.
        assert_eq!(output.exit_code, 0);
        assert!(output.stdout.contains("hello from rhai"));
    }

    #[test]
    fn with_engine_uses_injected_rhai_runtime() {
        let mut rhai = Engine::new();
        rhai.register_fn("answer", || 42_i64);
        let engine = RhaiEngine::with_engine(rhai);

        let output = engine.run(ScriptInput {
            source: "answer()".into(),
            lang: ScriptLang::Rhai,
            vars: BTreeMap::new(),
            policy: SandboxPolicy::permissive(),
            timeout_secs: 0,
        });

        // The custom host function proves this instance uses the injected Rhai runtime.
        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.vars.get("_result").and_then(|v| v.as_i64()),
            Some(42)
        );
    }

    #[test]
    fn register_rhai_engine_adds_default_engine_to_registry() {
        let registry = rhai_engine_registry();

        assert_eq!(registry.languages(), vec![ScriptLang::Rhai]);
        let output = registry
            .get(ScriptLang::Rhai)
            .expect("rhai engine should be registered")
            .run(ScriptInput {
                source: "21 * 2".into(),
                lang: ScriptLang::Rhai,
                vars: BTreeMap::new(),
                policy: SandboxPolicy::permissive(),
                timeout_secs: 0,
            });

        // Registry helpers should expose a usable default Rhai implementation, not only a marker.
        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.vars.get("_result").and_then(|v| v.as_i64()),
            Some(42)
        );
    }

    #[test]
    fn rhai_engine_factory_builds_rhai_engine_instances() {
        let factory = RhaiEngineFactory;
        assert_eq!(factory.lang(), ScriptLang::Rhai);

        let engine = factory.build();
        let output = engine.run(ScriptInput {
            source: "6 * 7".into(),
            lang: ScriptLang::Rhai,
            vars: BTreeMap::new(),
            policy: SandboxPolicy::permissive(),
            timeout_secs: 0,
        });

        assert_eq!(output.exit_code, 0);
        assert_eq!(
            output.vars.get("_result").and_then(|v| v.as_i64()),
            Some(42)
        );
    }
}

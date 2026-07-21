//! 可嵌入脚本引擎的通用契约。
//!
//! 本模块只定义脚本运行请求、输出、引擎 trait、工厂和注册表。具体 Rhai、Python、
//! Bash 等运行器放在各自实现 crate 中，通过 [`ScriptEngineFactory`] 注入宿主。

use az_sandbox::sandbox::SandboxPolicy;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

// ─── Script Types ───────────────────────────────────────────────────

/// 平台当前识别的脚本语言。
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, strum::Display, strum::EnumString, strum::IntoStaticStr, strum::VariantArray)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum ScriptLang {
    Curl,
    Rhai,
    Python,
    TypeScript,
    Bash,
}

impl ScriptLang {
    #[allow(dead_code)]
    pub const ALL: &'static [Self] = <Self as strum::VariantArray>::VARIANTS;

    #[must_use]
    pub fn as_str(self) -> &'static str {
        self.into()
    }

    #[must_use]
    pub fn code(self) -> &'static str {
        self.as_str()
    }

    pub fn from_code(value: &str) -> Option<Self> {
        value.parse().ok()
    }
}

/// 一次脚本执行的输入。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScriptInput {
    /// 脚本源码。
    pub source: String,
    /// 脚本语言。
    pub lang: ScriptLang,
    /// 注入脚本作用域的变量。
    pub vars: BTreeMap<String, serde_json::Value>,
    /// 本次执行使用的沙箱策略。
    pub policy: SandboxPolicy,
    /// 执行超时时间，单位秒；`0` 表示不设置超时。
    pub timeout_secs: u64,
}

/// 一次脚本执行的输出。
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ScriptOutput {
    /// 进程式执行语义下的退出码，`0` 表示成功。
    pub exit_code: i32,
    /// 捕获到的标准输出。
    pub stdout: String,
    /// 捕获到的标准错误。
    pub stderr: String,
    /// 脚本执行后导出的变量。
    pub vars: BTreeMap<String, serde_json::Value>,
    /// 执行耗时，单位毫秒。
    pub duration_ms: u64,
}

// ─── Engine Trait ───────────────────────────────────────────────────

/// 脚本异步执行方法返回的 boxed future。
pub type ScriptFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// 统一脚本引擎接口。
///
/// 宿主只依赖这个 trait 调度脚本；具体实现可以来自内置 crate，也可以来自插件工厂。
pub trait ScriptEngine: Send + Sync {
    /// 当前引擎负责的脚本语言。
    fn lang(&self) -> ScriptLang;

    /// 同步执行脚本，会阻塞当前调用线程。
    ///
    /// 异步宿主应优先调用 [`ScriptEngine::run_async`]。
    fn run(&self, input: ScriptInput) -> ScriptOutput;

    /// 异步执行脚本。
    fn run_async<'a>(&'a self, input: ScriptInput) -> ScriptFuture<'a, ScriptOutput>;
}

/// boxed 脚本引擎 trait object。
pub type BoxScriptEngine = Box<dyn ScriptEngine>;

/// 脚本引擎 provider 的工厂边界。
///
/// 插件 crate 或宿主适配层通过工厂交付新引擎实例，避免注册表持有具体实现类型。
pub trait ScriptEngineFactory: Send + Sync {
    /// 当前工厂生产的脚本语言。
    fn lang(&self) -> ScriptLang;

    /// 构建一个新的脚本引擎实例。
    fn build(&self) -> BoxScriptEngine;
}

// ─── Engine Registry ────────────────────────────────────────────────

/// 可用脚本引擎注册表。
pub trait ScriptEngineRegistry: Send + Sync {
    /// 注册或替换脚本引擎。
    fn register(&mut self, engine: BoxScriptEngine);

    /// 按语言获取脚本引擎。
    fn get(&self, lang: ScriptLang) -> Option<&dyn ScriptEngine>;

    /// 列出已注册脚本语言。
    fn languages(&self) -> Vec<ScriptLang>;
}

/// 面向直接组合场景的内存脚本引擎注册表。
#[derive(Default)]
pub struct InMemoryScriptEngineRegistry {
    engines: Vec<Box<dyn ScriptEngine>>,
}

impl InMemoryScriptEngineRegistry {
    /// 创建空注册表。
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用已有引擎实例创建注册表。
    #[must_use]
    pub fn with_engines(engines: Vec<BoxScriptEngine>) -> Self {
        let mut registry = Self::new();
        for engine in engines {
            registry.register(engine);
        }
        registry
    }

    /// 通过一组工厂创建注册表。
    #[must_use]
    pub fn with_factories(factories: &[&dyn ScriptEngineFactory]) -> Self {
        let mut registry = Self::new();
        for factory in factories {
            registry.register_from_factory(*factory);
        }
        registry
    }

    /// 通过工厂契约构建并注册一个引擎。
    pub fn register_from_factory(&mut self, factory: &dyn ScriptEngineFactory) {
        register_engine_factory(self, factory);
    }
}

/// 通过工厂构建引擎，并注册到任意脚本引擎注册表。
pub fn register_engine_factory(
    registry: &mut dyn ScriptEngineRegistry,
    factory: &dyn ScriptEngineFactory,
) {
    let expected_lang = factory.lang();
    let engine = factory.build();
    debug_assert_eq!(
        engine.lang(),
        expected_lang,
        "script engine factory returned an engine for a different language"
    );
    registry.register(engine);
}

impl ScriptEngineRegistry for InMemoryScriptEngineRegistry {
    fn register(&mut self, engine: BoxScriptEngine) {
        let lang = engine.lang();
        if let Some(slot) = self
            .engines
            .iter_mut()
            .find(|current| current.lang() == lang)
        {
            *slot = engine;
        } else {
            self.engines.push(engine);
        }
    }

    fn get(&self, lang: ScriptLang) -> Option<&dyn ScriptEngine> {
        self.engines
            .iter()
            .find(|engine| engine.lang() == lang)
            .map(|engine| engine.as_ref())
    }

    fn languages(&self) -> Vec<ScriptLang> {
        ScriptLang::ALL
            .iter()
            .copied()
            .filter(|lang| self.get(*lang).is_some())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BoxScriptEngine, InMemoryScriptEngineRegistry, ScriptEngine, ScriptEngineFactory,
        ScriptEngineRegistry, ScriptFuture, ScriptInput, ScriptLang, ScriptOutput,
    };
    use std::collections::BTreeMap;

    struct StaticEngine {
        lang: ScriptLang,
        result: &'static str,
    }

    struct StaticEngineFactory {
        lang: ScriptLang,
        result: &'static str,
    }

    impl ScriptEngineFactory for StaticEngineFactory {
        fn lang(&self) -> ScriptLang {
            self.lang
        }

        fn build(&self) -> BoxScriptEngine {
            Box::new(StaticEngine {
                lang: self.lang,
                result: self.result,
            })
        }
    }

    impl ScriptEngine for StaticEngine {
        fn lang(&self) -> ScriptLang {
            self.lang
        }

        fn run(&self, _input: ScriptInput) -> ScriptOutput {
            ScriptOutput {
                exit_code: 0,
                stdout: self.result.to_string(),
                stderr: String::new(),
                vars: BTreeMap::new(),
                duration_ms: 0,
            }
        }

        fn run_async<'a>(&'a self, input: ScriptInput) -> ScriptFuture<'a, ScriptOutput> {
            Box::pin(async move { self.run(input) })
        }
    }

    #[test]
    fn script_lang_codes_follow_lowercase_wires() {
        // 语言 code 是 API、插件清单和持久化配置之间的稳定 wire 值。
        assert_eq!(ScriptLang::TypeScript.code(), "typescript");
        assert_eq!(ScriptLang::from_code("bash"), Some(ScriptLang::Bash));
        assert_eq!(
            serde_json::to_string(&ScriptLang::Python).expect("serialize"),
            "\"python\""
        );
    }

    #[test]
    fn in_memory_registry_registers_replaces_and_lists_stably() {
        let mut registry =
            InMemoryScriptEngineRegistry::with_engines(vec![Box::new(StaticEngine {
                lang: ScriptLang::Bash,
                result: "bash",
            })]);
        registry.register(Box::new(StaticEngine {
            lang: ScriptLang::Rhai,
            result: "rhai-v1",
        }));
        registry.register(Box::new(StaticEngine {
            lang: ScriptLang::Rhai,
            result: "rhai-v2",
        }));

        // 注册同语言引擎时应替换旧实例，同时语言列表保持 enum 声明顺序。
        assert_eq!(
            registry.languages(),
            vec![ScriptLang::Rhai, ScriptLang::Bash]
        );
        assert_eq!(
            registry
                .get(ScriptLang::Rhai)
                .expect("rhai engine should be registered")
                .run(ScriptInput {
                    source: String::new(),
                    lang: ScriptLang::Rhai,
                    vars: BTreeMap::new(),
                    policy: az_sandbox::sandbox::SandboxPolicy::permissive(),
                    timeout_secs: 0,
                })
                .stdout,
            "rhai-v2"
        );
    }

    #[test]
    fn in_memory_registry_builds_from_engine_factories() {
        let bash = StaticEngineFactory {
            lang: ScriptLang::Bash,
            result: "bash",
        };
        let rhai = StaticEngineFactory {
            lang: ScriptLang::Rhai,
            result: "rhai",
        };

        let registry = InMemoryScriptEngineRegistry::with_factories(&[&bash, &rhai]);

        // 工厂注入应与直接注册保持一致，宿主不需要知道具体引擎类型。
        assert_eq!(
            registry.languages(),
            vec![ScriptLang::Rhai, ScriptLang::Bash]
        );
        assert_eq!(
            registry
                .get(ScriptLang::Bash)
                .expect("bash engine should be registered")
                .run(ScriptInput {
                    source: String::new(),
                    lang: ScriptLang::Bash,
                    vars: BTreeMap::new(),
                    policy: az_sandbox::sandbox::SandboxPolicy::permissive(),
                    timeout_secs: 0,
                })
                .stdout,
            "bash"
        );
    }

    #[test]
    fn register_from_factory_replaces_existing_engine_for_same_lang() {
        let mut registry = InMemoryScriptEngineRegistry::new();
        let first = StaticEngineFactory {
            lang: ScriptLang::Rhai,
            result: "first",
        };
        let second = StaticEngineFactory {
            lang: ScriptLang::Rhai,
            result: "second",
        };

        registry.register_from_factory(&first);
        registry.register_from_factory(&second);

        // 同一语言只能保留一个活跃引擎，后注册的工厂覆盖旧实例。
        assert_eq!(registry.languages(), vec![ScriptLang::Rhai]);
        assert_eq!(
            registry
                .get(ScriptLang::Rhai)
                .expect("rhai engine should be registered")
                .run(ScriptInput {
                    source: String::new(),
                    lang: ScriptLang::Rhai,
                    vars: BTreeMap::new(),
                    policy: az_sandbox::sandbox::SandboxPolicy::permissive(),
                    timeout_secs: 0,
                })
                .stdout,
            "second"
        );
    }
}

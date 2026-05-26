//! Common contracts for embeddable script engines.

use az_derive_aliases::{apply, error_eq, plain_default, serde_eq, serde_lower_code_enum};
use az_sandbox::sandbox::SandboxPolicy;
use std::collections::BTreeMap;
use std::future::Future;
use std::pin::Pin;

// ─── Script Types ───────────────────────────────────────────────────

/// Supported script languages.
#[apply(serde_lower_code_enum)]
pub enum ScriptLang {
    Curl,
    Rhai,
    Python,
    TypeScript,
    Bash,
}

/// Input to a script execution.
#[apply(serde_eq)]
pub struct ScriptInput {
    /// The script source code.
    pub source: String,
    /// Script language.
    pub lang: ScriptLang,
    /// Variables passed into the script scope.
    pub vars: BTreeMap<String, serde_json::Value>,
    /// Sandbox policy for this execution.
    pub policy: SandboxPolicy,
    /// Timeout in seconds (0 = no timeout).
    pub timeout_secs: u64,
}

/// Output from a script execution.
#[apply(serde_eq)]
pub struct ScriptOutput {
    /// Exit code (0 = success).
    pub exit_code: i32,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Variables exported from the script scope.
    pub vars: BTreeMap<String, serde_json::Value>,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
}

// ─── Engine Trait ───────────────────────────────────────────────────

/// Future returned by script execution methods.
pub type ScriptFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// Unified script engine trait.
///
/// Implementations: RhaiEngine, PythonEngine, TypeScriptEngine, BashEngine.
pub trait ScriptEngine: Send + Sync {
    /// The language this engine handles.
    fn lang(&self) -> ScriptLang;

    /// Execute a script synchronously (blocking the calling thread).
    /// For async execution, use `run_async`.
    fn run(&self, input: ScriptInput) -> ScriptOutput;

    /// Execute a script asynchronously.
    fn run_async<'a>(&'a self, input: ScriptInput) -> ScriptFuture<'a, ScriptOutput>;
}

// ─── Engine Registry ────────────────────────────────────────────────

/// A registry of all available script engines.
pub trait ScriptEngineRegistry: Send + Sync {
    /// Register a script engine.
    fn register(&mut self, engine: Box<dyn ScriptEngine>);

    /// Get an engine for the given language.
    fn get(&self, lang: ScriptLang) -> Option<&dyn ScriptEngine>;

    /// List all registered languages.
    fn languages(&self) -> Vec<ScriptLang>;
}

/// In-memory script engine registry for hosts that compose engines directly.
#[apply(plain_default)]
pub struct InMemoryScriptEngineRegistry {
    engines: Vec<Box<dyn ScriptEngine>>,
}

impl InMemoryScriptEngineRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a registry preloaded with engine implementations.
    #[must_use]
    pub fn with_engines(engines: Vec<Box<dyn ScriptEngine>>) -> Self {
        let mut registry = Self::new();
        for engine in engines {
            registry.register(engine);
        }
        registry
    }
}

impl ScriptEngineRegistry for InMemoryScriptEngineRegistry {
    fn register(&mut self, engine: Box<dyn ScriptEngine>) {
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

// ─── Error ──────────────────────────────────────────────────────────

#[apply(error_eq)]
pub enum ScriptError {
    #[error("unsupported language: {0:?}")]
    UnsupportedLanguage(ScriptLang),

    #[error("script execution timed out after {0}s")]
    Timeout(u64),

    #[error("sandbox policy violation: {0}")]
    SandboxViolation(String),

    #[error("engine error: {0}")]
    Engine(String),
}

#[cfg(test)]
mod tests {
    use super::{
        InMemoryScriptEngineRegistry, ScriptEngine, ScriptEngineRegistry, ScriptFuture,
        ScriptInput, ScriptLang, ScriptOutput,
    };
    use std::collections::BTreeMap;

    struct StaticEngine {
        lang: ScriptLang,
        result: &'static str,
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
}

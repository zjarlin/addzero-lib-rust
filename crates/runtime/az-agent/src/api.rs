//! agent 运行时 crate 的公开 API 探针。

/// agent 运行时依赖的编译期标记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DependencyMarkers {
    /// OpenAI client 依赖标记。
    pub async_openai_client: &'static str,
    /// Tokio runtime 依赖标记。
    pub tokio_runtime: &'static str,
    /// tracing subscriber 依赖标记。
    pub tracing_subscriber: &'static str,
}

/// 返回类型名标记，用于证明 agent 运行时依赖已经接入。
pub fn dependency_markers() -> DependencyMarkers {
    DependencyMarkers {
        async_openai_client: std::any::type_name::<
            async_openai::Client<async_openai::config::OpenAIConfig>,
        >(),
        tokio_runtime: std::any::type_name::<tokio::runtime::Runtime>(),
        tracing_subscriber: std::any::type_name::<tracing_subscriber::FmtSubscriber>(),
    }
}

//! Public API probes for the agent runtime crate.

/// Compile-time markers for agent runtime dependencies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DependencyMarkers {
    /// Marker for the OpenAI client dependency.
    pub async_openai_client: &'static str,
    /// Marker for the Tokio runtime dependency.
    pub tokio_runtime: &'static str,
    /// Marker for the tracing subscriber dependency.
    pub tracing_subscriber: &'static str,
}

/// Returns type-name markers proving the agent runtime dependencies are wired.
pub fn dependency_markers() -> DependencyMarkers {
    DependencyMarkers {
        async_openai_client: std::any::type_name::<
            async_openai::Client<async_openai::config::OpenAIConfig>,
        >(),
        tokio_runtime: std::any::type_name::<tokio::runtime::Runtime>(),
        tracing_subscriber: std::any::type_name::<tracing_subscriber::FmtSubscriber>(),
    }
}

use az_agent::api::dependency_markers;

#[test]
fn dependency_markers_should_expose_agent_runtime_dependencies() {
    let markers = dependency_markers();

    assert!(markers.async_openai_client.contains("async_openai"));
    assert!(markers.tokio_runtime.contains("tokio"));
    assert!(markers.tracing_subscriber.contains("tracing_subscriber"));
}

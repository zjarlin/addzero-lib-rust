use az_agent_runtime_contract::{AgentArtifactChannel, AgentNodeStatus, PairingStatus};

#[test]
fn contract_enums_keep_snake_case_wire_shape() {
    assert_eq!(AgentArtifactChannel::MacosBinary.as_str(), "macos_binary");
    assert_eq!(
        AgentArtifactChannel::from_code("docker_compose"),
        Some(AgentArtifactChannel::DockerCompose)
    );
    assert_eq!(
        serde_json::to_string(&PairingStatus::Approved).expect("pairing status should serialize"),
        "\"approved\""
    );
    assert_eq!(
        serde_json::to_string(&AgentNodeStatus::Offline).expect("node status should serialize"),
        "\"offline\""
    );
}

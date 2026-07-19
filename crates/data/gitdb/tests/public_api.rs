use az_gitdb::config::{GitDbLoadBalanceStrategy, GitDbNodeConfig, GitDbNodeRole};

#[test]
fn node_role_code_is_snake_case() {
    assert_eq!(GitDbNodeRole::ReadWrite.code(), "read_write");
}

#[test]
fn node_role_from_code_reads_snake_case() {
    assert_eq!(
        GitDbNodeRole::from_code("write_only"),
        Some(GitDbNodeRole::WriteOnly)
    );
}

#[test]
fn node_role_all_lists_supported_roles() {
    assert_eq!(
        GitDbNodeRole::ALL,
        &[
            GitDbNodeRole::ReadWrite,
            GitDbNodeRole::ReadOnly,
            GitDbNodeRole::WriteOnly,
        ]
    );
}

#[test]
fn load_balance_strategy_default_stays_round_robin() {
    assert_eq!(
        GitDbLoadBalanceStrategy::default(),
        GitDbLoadBalanceStrategy::RoundRobin
    );
}

#[test]
fn load_balance_strategy_code_is_snake_case() {
    assert_eq!(
        GitDbLoadBalanceStrategy::LeastInFlight.code(),
        "least_in_flight"
    );
}

#[test]
fn node_config_uses_remote_repository_as_source() {
    let config = GitDbNodeConfig::new(
        "primary",
        "git@github.com:example/gitdb-data.git",
        "/tmp/gitdb-data",
    );

    assert_eq!(config.remote_url, "git@github.com:example/gitdb-data.git");
    assert_eq!(
        config.checkout_path,
        std::path::PathBuf::from("/tmp/gitdb-data")
    );
    assert!(config.clone_if_missing);
}

use az_gitdb::{GitDbLoadBalanceStrategy, GitDbNodeRole};

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

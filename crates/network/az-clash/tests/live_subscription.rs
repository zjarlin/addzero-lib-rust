use az_clash::{
    ClashResult, batch_speed_test, fetch_and_parse, generate_clash_config, select_fastest_node,
};
use std::time::Duration;

const SUBSCRIPTION_URL: &str = "https://dash.pqjc.site/api/v1/pq/b21996144d42b3fa6565c15c7fd18415";

#[tokio::test]
async fn live_subscription_should_fetch_test_and_generate_config() -> ClashResult<()> {
    let base_nodes = fetch_and_parse(SUBSCRIPTION_URL).await?;
    println!("base subscription nodes: {}", base_nodes.len());
    assert!(!base_nodes.is_empty());

    let clash_url = format!("{SUBSCRIPTION_URL}?flag=clash");
    let yaml_nodes = fetch_and_parse(&clash_url).await?;
    println!("clash yaml subscription nodes: {}", yaml_nodes.len());
    assert!(!yaml_nodes.is_empty());

    let results = batch_speed_test(&base_nodes, 10, Duration::from_secs(2)).await;

    println!("speed test results: {}", results.len());
    for result in results.iter().take(10) {
        println!(
            "node={} success={} latency_ms={:?} error={:?}",
            result.node_index, result.success, result.latency_ms, result.error_msg
        );
    }

    let selected = match select_fastest_node(&base_nodes, &results) {
        Ok(node) => node,
        Err(error) => {
            println!(
                "no successful live speed test, falling back to first node for config check: {error}"
            );
            &base_nodes[0]
        }
    };
    let config = generate_clash_config(selected, 7890)?;
    println!("selected node: {}", selected.name);
    println!("generated config bytes: {}", config.len());

    assert!(config.contains("proxy-groups:"));
    Ok(())
}

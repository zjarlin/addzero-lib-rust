use anyhow::Result;
use az_proxy::clash::generate_clash_config;
use az_proxy::fetcher::fetch_and_parse;
use az_proxy::selector::select_fastest_node;
use az_proxy::speedtest::batch_speed_test;
use std::time::Duration;

#[tokio::test]
#[ignore = "requires a live proxy subscription URL in AZ_PROXY_LIVE_SUBSCRIPTION_URL"]
async fn live_subscription_should_fetch_test_and_generate_config() -> Result<()> {
    let subscription_url = std::env::var("AZ_PROXY_LIVE_SUBSCRIPTION_URL")
        .expect("AZ_PROXY_LIVE_SUBSCRIPTION_URL is required");
    let subscription_url = subscription_url.as_str();

    let base_nodes = fetch_and_parse(subscription_url).await?;
    println!("base subscription nodes: {}", base_nodes.len());
    assert!(!base_nodes.is_empty());

    let clash_url = format!("{subscription_url}?flag=clash");
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

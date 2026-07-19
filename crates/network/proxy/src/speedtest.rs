use crate::types::{DEFAULT_SPEEDTEST_CONCURRENCY, DEFAULT_SPEEDTEST_TIMEOUT, ProxyNode, SpeedTestResult};
use std::cmp::Ordering;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Semaphore;
use tokio::task::JoinSet;

/// 对每个代理节点执行 TCP 连接延迟测试。
///
/// `concurrency` 为 `0` 时回退到 [`DEFAULT_SPEEDTEST_CONCURRENCY`]；
/// `timeout` 为 `0ms` 时回退到 [`DEFAULT_SPEEDTEST_TIMEOUT`]。
/// 返回结果按成功连接的延迟从低到高排序，失败结果排在成功结果之后。
pub async fn batch_speed_test(
    nodes: &[ProxyNode],
    concurrency: usize,
    timeout: Duration,
) -> Vec<SpeedTestResult> {
    let concurrency = if concurrency == 0 {
        DEFAULT_SPEEDTEST_CONCURRENCY
    } else {
        concurrency
    };
    let timeout = if timeout.is_zero() {
        DEFAULT_SPEEDTEST_TIMEOUT
    } else {
        timeout
    };
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let mut join_set = JoinSet::new();

    for (node_index, node) in nodes.iter().enumerate() {
        let permit_source = Arc::clone(&semaphore);
        let server = node.server.clone();
        let port = node.port;
        join_set.spawn(async move {
            let permit = permit_source.acquire_owned().await;
            if let Err(error) = permit {
                return SpeedTestResult {
                    node_index,
                    latency_ms: None,
                    success: false,
                    error_msg: Some(format!("speed test semaphore closed: {error}")),
                };
            }

            test_tcp_latency(node_index, server, port, timeout).await
        });
    }

    let mut results = Vec::with_capacity(nodes.len());
    while let Some(joined) = join_set.join_next().await {
        match joined {
            Ok(result) => results.push(result),
            Err(error) => results.push(SpeedTestResult {
                node_index: usize::MAX,
                latency_ms: None,
                success: false,
                error_msg: Some(format!("speed test task failed: {error}")),
            }),
        }
    }

    results.sort_by(compare_speed_results);
    results
}

async fn test_tcp_latency(
    node_index: usize,
    server: String,
    port: u16,
    timeout_duration: Duration,
) -> SpeedTestResult {
    let address = format!("{server}:{port}");
    let started_at = Instant::now();
    match tokio::time::timeout(timeout_duration, TcpStream::connect(&address)).await {
        Ok(Ok(_stream)) => SpeedTestResult {
            node_index,
            latency_ms: Some(started_at.elapsed().as_millis()),
            success: true,
            error_msg: None,
        },
        Ok(Err(error)) => SpeedTestResult {
            node_index,
            latency_ms: None,
            success: false,
            error_msg: Some(format!("tcp connect failed for {address}: {error}")),
        },
        Err(_elapsed) => SpeedTestResult {
            node_index,
            latency_ms: None,
            success: false,
            error_msg: Some(format!("timeout after {}ms for {address}", timeout_duration.as_millis())),
        },
    }
}

fn compare_speed_results(left: &SpeedTestResult, right: &SpeedTestResult) -> Ordering {
    match (left.success, right.success) {
        (true, true) => left
            .latency_ms
            .unwrap_or(u128::MAX)
            .cmp(&right.latency_ms.unwrap_or(u128::MAX))
            .then_with(|| left.node_index.cmp(&right.node_index)),
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (false, false) => left.node_index.cmp(&right.node_index),
    }
}

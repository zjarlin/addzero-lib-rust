use crate::types::{ProxyError, ProxyNode, ProxyResult, SpeedTestResult};

/// Selects the fastest successful node according to sorted speed test results.
///
/// # Errors
///
/// Returns [`ProxyError::NoSuccessfulSpeedTest`] when none of the results
/// succeeded or all successful results point outside the node slice.
pub fn select_fastest_node<'a>(
    nodes: &'a [ProxyNode],
    results: &[SpeedTestResult],
) -> ProxyResult<&'a ProxyNode> {
    results
        .iter()
        .filter(|result| result.success)
        .filter_map(|result| nodes.get(result.node_index))
        .next()
        .ok_or(ProxyError::NoSuccessfulSpeedTest)
}

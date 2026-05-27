use crate::types::{ProxyError, ProxyNode, ProxyResult, SpeedTestResult};

/// 根据排序后的测速结果选择最快的成功节点。
///
/// # Errors
///
/// 当没有成功测速结果，或成功结果全部指向节点列表之外时，返回
/// [`ProxyError::NoSuccessfulSpeedTest`]。
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

use std::sync::Arc;

use rudi::Context;

use crate::responses::{ResponsesResult, ResponsesRunRequest};

/// 兼容 Responses 的 agent 运行时 SPI。
#[async_trait::async_trait]
pub trait AgentResponsesSpi: Send + Sync {
    /// 通过注入的 agent provider 运行 Responses 风格请求。
    async fn run_responses(&self, request: ResponsesRunRequest) -> anyhow::Result<ResponsesResult>;
}

/// 存入 Rudi 的 agent 运行时 SPI 共享指针类型。
pub type AgentResponsesSpiRef = Arc<dyn AgentResponsesSpi + Send + Sync>;

/// 从 Rudi 上下文解析 Responses agent SPI；缺失时返回边界错误。
pub fn resolve_agent_responses_spi(cx: &mut Context) -> anyhow::Result<AgentResponsesSpiRef> {
    cx.resolve_option::<AgentResponsesSpiRef>().ok_or_else(|| {
        anyhow::anyhow!(
            "missing Rudi provider for `AgentResponsesSpiRef`; inject `Arc<dyn AgentResponsesSpi + Send + Sync>` before running az-agent"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_missing_spi_provider() {
        let mut cx = Context::default();

        let error = match resolve_agent_responses_spi(&mut cx) {
            Ok(_) => panic!("expected missing SPI provider error"),
            Err(error) => error,
        };

        assert!(
            error
                .to_string()
                .contains("missing Rudi provider for `AgentResponsesSpiRef`")
        );
    }
}

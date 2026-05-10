//! Clash 订阅解析、代理节点测试和最小配置生成。
//!
//! `az-clash` 是一个用于处理 Clash 兼容订阅数据的独立库。
//! 它获取订阅 URL、解析 Clash YAML 和常见代理 URI 格式、
//! 测试节点 TCP 延迟，并为选定节点生成最小 Clash 配置。
//! 它不调用也不依赖 Clash Verge 或任何 Clash 二进制文件。
//!
//! # 示例
//!
//! ```no_run
//! # async fn run() -> az_clash::ClashResult<()> {
//! let config = az_clash::select_fastest("https://example.com/subscription", 10).await?;
//! println!("{config}");
//! # Ok(())
//! # }
//! ```

#![deny(missing_docs)]

automod::dir!("src");

pub mod parser;

pub use fetcher::{FetchedSubscription, fetch_and_parse, fetch_subscription};
pub use parser::{parse_clash_yaml, parse_proxy_uri, parse_subscription, parse_uri_lines};
pub use selector::{generate_clash_config, select_fastest, select_fastest_node};
pub use speedtest::batch_speed_test;
pub use types::{
    ClashConfig, ClashError, ClashResult, DEFAULT_MIXED_PORT, DEFAULT_SPEEDTEST_CONCURRENCY,
    DEFAULT_SPEEDTEST_TIMEOUT, ProxyGroup, ProxyNode, ProxyType, SpeedTestResult,
};

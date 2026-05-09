//! Clash subscription parsing, proxy node testing, and minimal config generation.
//!
//! `az-clash` is a standalone library for working with Clash-compatible
//! subscription data. It fetches subscription URLs, parses Clash YAML and common
//! proxy URI formats, tests node TCP latency, and emits a minimal Clash config
//! for a selected node. It does not invoke or depend on Clash Verge or any Clash
//! binary.
//!
//! # Example
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

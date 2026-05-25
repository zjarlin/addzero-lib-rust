//! 将 curl 命令字符串解析为结构化 HTTP 请求，并支持直接执行。
//!
//! `az-curl` 的公开面保持很窄：[`parse_curl`] 负责解析，[`execute_curl`]
//! 负责执行。库错误使用结构化 [`CurlError`]，不向外暴露 `anyhow::Error`。
//!
//! # 示例
//!
//! ```
//! use az_curl::parse_curl;
//!
//! # fn main() -> az_curl::CurlResult<()> {
//! let parsed = parse_curl(r#"curl -H "Accept: application/json" https://api.example.com"#)?;
//! assert_eq!(parsed.url, "https://api.example.com");
//! # Ok(())
//! # }
//! ```

automod::dir!("src");

pub use error::{CurlError, CurlResult};
pub use execute::{CurlResponse, execute_curl};
pub use model::ParsedCurl;
pub use parse::parse_curl;

#[macro_export]
macro_rules! curl {
    ($command:expr) => {
        $crate::parse_curl($command)
    };
}

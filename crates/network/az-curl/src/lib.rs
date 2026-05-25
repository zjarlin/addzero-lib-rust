#![doc = include_str!("../README.md")]

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

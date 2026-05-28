#![doc = include_str!("../README.md")]

pub mod common {
    pub mod error;
    pub mod http;
}

pub mod error {
    pub use crate::common::error::*;
}

mod http {
    pub(crate) use crate::common::http::*;
}

pub mod dogsms {
    pub mod client;
}

pub mod grizzlysms {
    pub mod client;
}

pub mod model;
pub mod provider;

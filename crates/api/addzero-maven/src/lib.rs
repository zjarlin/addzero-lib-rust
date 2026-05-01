#![forbid(unsafe_code)]

mod config;
mod error;
mod http;
mod maven;
mod util;

pub use config::{ApiConfig, ApiConfigBuilder};
pub use error::{
    CreatesError, CreatesError as MavenError, CreatesResult, CreatesResult as MavenResult,
};
pub use maven::{MavenArtifact, MavenCentralApi, create_maven_central_api};

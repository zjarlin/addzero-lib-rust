#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod auth;
mod config;
mod error;
mod provider;

pub use auth::{
    AuthDiscovery, AuthDiscoveryOptions, AuthLoginFlow, AuthMethod, AuthSession, AuthState,
    CommandOutput, CommandRunner, GitHostingAccountStatus, SystemCommandRunner,
};
pub use config::{GitAccountConfig, GitAccountConfigStore};
pub use error::{AzGitError, Result};
pub use provider::{GitHostingProvider, GitHostingProviderInfo};

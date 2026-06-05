#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

mod auth;
mod config;
mod error;
mod provider;
mod repository;

pub use auth::{
    AuthDiscovery, AuthDiscoveryOptions, AuthLoginFlow, AuthMethod, AuthSession, AuthState,
    CommandOutput, CommandRunner, GitHostingAccountStatus, SystemCommandRunner,
};
pub use config::{
    DEFAULT_SYNC_WORKSPACE, GitAccountConfig, GitAccountConfigStore, GitProjectBinding,
};
pub use error::{AzGitError, Result};
pub use provider::{GitHostingProvider, GitHostingProviderInfo};
pub use repository::{GitRemoteRepository, GitRepositoryDiscovery};

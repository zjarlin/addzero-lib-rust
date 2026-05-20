#![forbid(unsafe_code)]
//! Provider-neutral OAuth2 helpers for installed apps, CLI tools, and device flows.
//!
//! The crate intentionally keeps OAuth mechanics separate from product-specific
//! clients. For Gmail verification-code reading, use [`GoogleOAuth2`] to obtain
//! a `gmail.readonly` access token, then pass that token to `az-gmail-code`.

mod client;
mod config;
mod error;
mod google;
mod loopback;
mod model;
mod pkce;
mod random;

pub use client::OAuth2Client;
pub use config::{AuthorizationCodeOptions, OAuth2Config, OAuth2ConfigBuilder};
pub use error::{OAuth2Error, OAuth2Result};
pub use google::GoogleOAuth2;
pub use loopback::{LoopbackAuthorizationSession, OAuth2AuthorizationCallback};
pub use model::{
    OAuth2DeviceAuthorization, OAuth2DeviceTokenPoll, OAuth2TokenResponse, OAuth2TokenSuccess,
};
pub use pkce::{PkcePair, generate_pkce_pair, generate_state};

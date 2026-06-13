#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!("src");

pub use client::OAuth2Client;
pub use config::{AuthorizationCodeOptions, OAuth2Config, OAuth2ConfigBuilder};
pub use error::{OAuth2Error, OAuth2Result};
pub use google::GoogleOAuth2;
pub use loopback::{LoopbackAuthorizationSession, OAuth2AuthorizationCallback};
pub use model::{
    OAuth2DeviceAuthorization, OAuth2DeviceTokenPoll, OAuth2TokenResponse, OAuth2TokenSuccess,
};
pub use pkce::{PkcePair, generate_pkce_pair, generate_state};

//! System-domain contracts for the AZ AIO admin platform.
//!
//! The shape mirrors the useful boundaries in `yudao-module-system`: identity,
//! organization, dictionary, menu, audit, tenant, messaging, OAuth2, and social
//! integration are treated as one system domain with explicit feature cuts.

automod::dir!(pub "src/system");

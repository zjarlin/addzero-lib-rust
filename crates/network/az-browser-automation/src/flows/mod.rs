//! Concrete registration flow adapters.
//!
//! Only implement flows for systems where the caller has authorization to run
//! account-creation automation.

pub mod kiro;

pub use kiro::KiroRegistrationFlow;

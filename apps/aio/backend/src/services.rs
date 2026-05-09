use std::future::Future;
use std::pin::Pin;

automod::dir!(pub "src/services");

/// Canonical boxed future alias for service trait methods.
pub type LocalBoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + 'a>>;

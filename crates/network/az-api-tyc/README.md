# az-api-tyc

Blocking Rust client and response models for the Tianyancha mini-program HTTP API surface that used to live in the JVM `tool-api-tyc` module.

The crate intentionally requires explicit credentials or `TYC_AUTHORIZATION` / `TYC_X_AUTH_TOKEN` environment variables. The old JVM module contained a hard-coded sample token; this Rust port does not preserve that credential-like historical baggage.

```rust,no_run
use az_api_tyc::client::TycApi;

# fn main() -> anyhow::Result<()> {
let api = TycApi::from_env()?;
let matches = api.search_company("中洛佳")?;
println!("{}", matches.company_total);
# Ok(())
# }
```

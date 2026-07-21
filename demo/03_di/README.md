# Rudi payment DI demo

This workspace shows a payment example built with Rudi dependency injection:

- `pay-api` declares the main trait `PayInterface`
- each plugin crate registers one payment implementation with `#[rudi::Transient]`
- the main app aggregates plugin registration with `rudi::enable!` and resolves them as `Vec<Box<dyn PayInterface>>`

Run:

```sh
cargo test
```

Usage:

```rust
use pay_api::{pay_interfaces, PayInterface, PayRequest};
use rudi::Context;

let request = PayRequest::new("demo-order", 100);
app::enable();
let mut context = Context::auto_register();
let pay_interfaces: Vec<Box<dyn PayInterface>> = pay_interfaces(&mut context);

pay_interfaces.into_iter().for_each(|pay| pay.pay(&request));
```

Important boundary:

- Rudi's automatic provider registration uses compile/link-time registration, not runtime discovery from crates.io.
- The app aggregates every plugin's `enable()` function before creating its context.
- Runtime order data is passed into the payment behavior when the app invokes it.

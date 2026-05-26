# inventory payment demo

This workspace shows a payment example built with `inventory` plugin registration:

- `pay-api` declares the main trait `PayInterface`
- each plugin crate registers one payment implementation factory
- the main app collects them as `Vec<Box<dyn PayInterface>>`

Run:

```sh
cargo test
```

Usage:

```rust
use inventory_plugin_alipay as _;
use inventory_plugin_wechat as _;
use inventory_plugin_other as _;
use pay_api::{PayInterface, PayRequest};

let request = PayRequest::new("demo-order", 100);
let pay_interfaces: Vec<Box<dyn PayInterface>> = pay_api::pay_interfaces();

pay_interfaces.into_iter().for_each(|pay| pay.pay(&request));
```

Important boundary:

- This is compile/link-time registration, not runtime discovery from crates.io.
- The app must depend on each plugin crate so the linker keeps those registrations.
- This shape registers factory functions into an `inventory` registry; runtime
  order data is passed into the payment behavior when the app invokes it.

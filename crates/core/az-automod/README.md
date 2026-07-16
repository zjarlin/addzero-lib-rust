# az-automod

`az-automod` is a directory-aware replacement for `automod`.

It keeps the familiar call shape:

```ignore
automod::dir!(pub "src");
```

When used as the workspace dependency alias below, existing code can continue to call `automod::dir!`:

```toml
[workspace.dependencies]
automod = { package = "az-automod", version = "2026.6.15" }
```

## Behavior

- Source files become modules, excluding `mod.rs`, `lib.rs`, and `main.rs`.
- Directories without a same-name `.rs` entry file become inline modules recursively.
- Directories with a same-name `.rs` entry file are skipped by the parent scan; the entry file stays responsible for its own nested module layout.
- `src/bin` is treated as Cargo binary source layout and is not collected as a module.
- Module names normalize `-` to `_`, and names beginning with digits are prefixed with `_`.
- On nightly Rust, `az-automod` asks rustc to track scanned directories with
  `proc_macro::tracked::path`, so adding or removing a source file can invalidate
  the macro expansion. On stable Rust this API is not available yet; after adding
  or deleting files, touch the file that calls `automod::dir!` or run a clean
  check if Cargo or the IDE keeps an old expansion.

For example:

```text
src/
  lib.rs
  payment/
    alipay.rs
    wechat.rs
  user.rs
```

```ignore
// src/lib.rs
automod::dir!(pub "src");
```

expands roughly to:

```ignore
pub mod payment {
    pub mod alipay;
    pub mod wechat;
}

pub mod user;
```

But this shape remains entry-file driven:

```text
src/
  lib.rs
  payment.rs
  payment/
    alipay.rs
    wechat.rs
```

The parent scan generates `pub mod payment;` from `payment.rs` and does not also inline `payment/`.

## License

This crate is derived from `automod` and remains dual licensed under MIT or Apache-2.0.

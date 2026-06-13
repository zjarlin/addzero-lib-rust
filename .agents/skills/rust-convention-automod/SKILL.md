---
name: rust-convention-automod
description: Enforce the addzero-lib-rust automod module layout. Use when adding, generating, or reorganizing Rust crate modules, especially when editing lib.rs, main.rs, api.rs, generated module trees, public module declarations, or replacing manual pub mod lists.
---

# Rust Convention Automod

Use `automod` as the default module collector for addzero-lib-rust crates.

## Default Shape

For crate roots, keep `src/lib.rs` focused:

```rust
#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)]

automod::dir!(pub "src");
```

For nested module directories, keep one file entrypoint and collect the matching directory:

```rust
//! Generated OpenAI REST API traits.

automod::dir!(pub "src/api");
```

Add `automod.workspace = true` to the touched crate when `automod::dir!` is introduced.

## Rules

- Prefer `automod::dir!(pub "src")` or `automod::dir!(pub "src/<module>")` over hand-maintained `pub mod foo;` lists.
- Do not create `mod.rs` by default. Use Rust 2018 file entrypoints such as `api.rs` plus `api/*.rs`.
- Keep `lib.rs` and `main.rs` as entrypoints only: crate docs, lint attributes, and module collection.
- Do not place experiments, drafts, backups, or generated trash under scanned source directories.
- Use module-level `pub use` only for deliberate public API shaping. It may coexist with `automod`; it must not replace module collection with a manual `pub mod` list.
- When generating many files, generate the files under a clean directory and use `automod` to collect them.
- After changing module layout, run `cargo fmt -p <crate>` and `cargo test -p <crate>` or at least `cargo check -p <crate>` when tests are expensive.

## Migration Steps

1. Add `automod.workspace = true` to the crate manifest if missing.
2. Replace manual module declarations in `lib.rs` or a directory entry file with `automod::dir!(pub "...")`.
3. Keep any intentional `pub use` lines needed for stable public names.
4. Run formatting and crate-level verification.

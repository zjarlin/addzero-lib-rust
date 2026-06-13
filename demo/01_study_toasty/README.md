# 01 Study Toasty

Small Toasty ORM study project using an in-memory SQLite database.

Run the Toasty ORM path:

```bash
cargo +1.95.0 run --manifest-path demo/01_study_toasty/Cargo.toml
```

Notes:

- `toasty 0.6.1` requires Rust `1.95`, so this demo and the repo toolchain pin use Rust `1.95.0`.
- No PostgreSQL URL is needed. The database is created in memory and discarded when the process exits.

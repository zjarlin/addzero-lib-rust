# 01 Study Toasty

Small PostgreSQL CRUD study project for Toasty only.

The real database URL is intentionally not stored in this directory. Export it before running:

```bash
export TOASTY_DATABASE_URL='postgresql://neondb_owner:...@ep-wandering-grass-anxhfczt-pooler.c-6.us-east-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require'
```

Run the Toasty ORM path:

```bash
cargo +1.94.0 run --manifest-path demo/01_study_toasty/Cargo.toml
```

Notes:

- Latest `toasty 0.6.1` currently requires Rust `1.95`; this repo default is Rust `1.92`, and this machine has `1.94.0` installed.
- This demo uses `toasty 0.4.0` because it has a PostgreSQL driver available and works with the installed `1.94.0` toolchain.
- With the Neon pooler URL, SQLx connects successfully in `../02_study_sqlx`; Toasty 0.4 currently fails against the same URL with `early eof`, likely in the older driver TLS/channel-binding path.


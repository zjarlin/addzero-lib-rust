# 01 Study Toasty

Small PostgreSQL CRUD study project for Toasty only.

The real database URL is intentionally not stored in this directory. Export it before running:

```bash
export TOASTY_DATABASE_URL='postgresql://neondb_owner:...@ep-wandering-grass-anxhfczt-pooler.c-6.us-east-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require'
```

Run the Toasty ORM path:

```bash
cargo +1.95.0 run --manifest-path demo/01_study_toasty/Cargo.toml
```

Notes:

- `toasty 0.6.1` requires Rust `1.95`, so this demo and the repo toolchain pin use Rust `1.95.0`.
- With the Neon pooler URL, SQLx connects successfully in `../02_study_sqlx`; keep Toasty driver behavior verified against the pinned Rust toolchain when changing versions.

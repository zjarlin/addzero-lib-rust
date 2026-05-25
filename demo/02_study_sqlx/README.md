# 02 Study SQLx

Small PostgreSQL CRUD study project for SQLx only.

The real database URL is intentionally not stored in this directory. Export it before running:

```bash
export SQLX_STUDY_DATABASE_URL='postgresql://neondb_owner:...@ep-wandering-grass-anxhfczt-pooler.c-6.us-east-1.aws.neon.tech/neondb?sslmode=require&channel_binding=require'
```

Run:

```bash
cargo run --manifest-path demo/02_study_sqlx/Cargo.toml
```

The program creates `demo_02_sqlx_todo`, inserts one row, reads it, updates it, deletes it, and prints each step.

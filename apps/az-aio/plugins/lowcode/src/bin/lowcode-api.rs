//! Standalone lowcode API server.
//! Run: cargo run -p lowcode --bin lowcode-api
//!
//! Set DATABASE_URL env var to enable persistence (SQLite or PostgreSQL).
//!   SQLite:  DATABASE_URL=sqlite://data/lowcode.db
//!   Without: uses in-memory store (data lost on restart)

use axum::Router;
use lowcode::backend::{
    routes::{LowcodeApiState, lowcode_router},
    store::LowcodeStore,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let database_url = std::env::var("DATABASE_URL").ok();

    let store = match database_url.as_deref() {
        Some(url) if !url.trim().is_empty() => {
            println!("Connecting to database: {url}");
            match LowcodeStore::new(url).await {
                Ok(s) => {
                    println!("Database connected. Seeding demo data...");
                    s.seed_demo();
                    println!("Seed complete.");
                    s
                }
                Err(e) => {
                    eprintln!("Database connection failed ({e}), falling back to in-memory");
                    let s = LowcodeStore::in_memory();
                    s.seed_demo();
                    s
                }
            }
        }
        _ => {
            println!("No DATABASE_URL set, using in-memory store");
            let s = LowcodeStore::in_memory();
            s.seed_demo();
            s
        }
    };

    lowcode::backend::record::RecordStore::global().seed_demo();

    let state = LowcodeApiState { store };
    let app = Router::new().merge(lowcode_router(state)).layer(
        CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any),
    );

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8081".into())
        .parse()?;
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    println!("Lowcode API listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}

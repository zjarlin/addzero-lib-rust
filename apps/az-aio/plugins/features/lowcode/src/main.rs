#[cfg(target_arch = "wasm32")]
fn main() {}

#[cfg(not(target_arch = "wasm32"))]
#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match native::run(std::env::args().skip(1).collect()).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::net::SocketAddr;

    use axum::{Router, routing::get};
    use az_aio_plugin_lowcode::{
        LowcodeApiState, LowcodeError, lowcode_api_router, resolve_lowcode_config,
    };
    use tokio::net::TcpListener;

    const DEFAULT_BIND_ADDR: &str = "127.0.0.1:8791";

    pub async fn run(args: Vec<String>) -> Result<(), LowcodeError> {
        let bind = parse_bind(args).parse::<SocketAddr>()?;
        let config = resolve_lowcode_config()?;
        let state = LowcodeApiState::connect(config).await?;
        let app = Router::new()
            .route("/health", get(|| async { "ok" }))
            .merge(lowcode_api_router(state));
        let listener = TcpListener::bind(bind)
            .await
            .map_err(|source| LowcodeError::Io {
                operation: "bind lowcode api",
                source,
            })?;
        let local_addr = listener.local_addr().map_err(|source| LowcodeError::Io {
            operation: "read lowcode api local address",
            source,
        })?;
        println!("az-aio lowcode serve");
        println!("config namespace: az-aio.dev");
        println!("database config key: lowcode.database_url");
        println!("listening: http://{local_addr}");
        println!("status: http://{local_addr}/api/lowcode/status");
        axum::serve(listener, app)
            .await
            .map_err(|source| LowcodeError::Io {
                operation: "serve lowcode api",
                source,
            })?;
        Ok(())
    }

    fn parse_bind(args: Vec<String>) -> String {
        let mut bind = DEFAULT_BIND_ADDR.to_string();
        let mut iter = args.into_iter();
        while let Some(arg) = iter.next() {
            if arg == "--bind" && let Some(value) = iter.next() {
                bind = value;
            }
        }
        bind
    }
}

mod native {
    automod::dir!(pub(super) "src/native");
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::process::ExitCode {
    match native::runtime::run(std::env::args().skip(1).collect()).await {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}

fn main() {
    if let Err(err) = aio_desktop::run() {
        eprintln!("{err:#}");
        std::process::exit(1);
    }
}

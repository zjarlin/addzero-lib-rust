fn main() {
    if let Err(err) = dotfiles::run() {
        eprintln!("error: {err}");
        std::process::exit(1);
    }
}

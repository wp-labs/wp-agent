fn main() {
    if let Err(err) = wist_upgrader::run() {
        eprintln!("{err}");
        std::process::exit(2);
    }
}

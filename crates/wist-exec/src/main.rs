fn main() {
    if let Err(err) = wist_exec::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

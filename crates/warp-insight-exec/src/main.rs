fn main() {
    if let Err(err) = warp_insight_exec::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

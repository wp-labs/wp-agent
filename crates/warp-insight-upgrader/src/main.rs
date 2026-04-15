fn main() {
    if let Err(err) = warp_insight_upgrader::run() {
        eprintln!("{err}");
        std::process::exit(2);
    }
}

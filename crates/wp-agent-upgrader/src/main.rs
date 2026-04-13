fn main() {
    if let Err(err) = wp_agent_upgrader::run() {
        eprintln!("{err}");
        std::process::exit(2);
    }
}

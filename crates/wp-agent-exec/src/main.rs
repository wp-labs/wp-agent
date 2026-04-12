fn main() {
    if let Err(err) = wp_agent_exec::run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

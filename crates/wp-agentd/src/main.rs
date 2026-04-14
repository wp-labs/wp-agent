fn main() {
    if let Err(err) = wp_agentd::run() {
        eprintln!("wp-agentd failed: {err}");
        std::process::exit(1);
    }
}

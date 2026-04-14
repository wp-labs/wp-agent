#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = wp_agentd::run().await {
        eprintln!("wp-agentd failed: {err}");
        std::process::exit(1);
    }
}

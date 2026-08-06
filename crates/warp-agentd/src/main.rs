#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = warp_agentd::run().await {
        eprintln!("warp-agentd failed: {err}");
        std::process::exit(1);
    }
}

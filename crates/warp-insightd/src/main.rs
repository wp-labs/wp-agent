#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Err(err) = warp_insightd::run().await {
        eprintln!("warp-insightd failed: {err}");
        std::process::exit(1);
    }
}

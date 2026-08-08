// @moju generated
// WarpInsightCenter 上级聚合控制中心服务：接收 WarpGateWay 状态上报。

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let config = warp_insight_center::config::CenterConfig::load_from_env()?;
    let store = warp_insight_center::infra::CenterStore::new(config.store_path.clone());
    store.seed(&config.gateway_credentials)?;
    let addr = config.listen_addr.clone();
    let app = warp_insight_center::api::router(config, store);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("warp-insight-center listening on http://{addr}");
    // 注入真实 peer 地址供限流按 IP 分桶（忽略可伪造的 x-real-ip / x-forwarded-for）。
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<std::net::SocketAddr>(),
    )
    .await?;
    Ok(())
}

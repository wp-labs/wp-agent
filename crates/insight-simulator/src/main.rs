use std::time::Duration;

use insight_simulator::{agentd, client, config, config::SimConfig, gateway};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match SimConfig::parse(&args) {
        Ok(config) => config,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    };
    let http_client = match client::build_client(config.insecure) {
        Ok(client) => client,
        Err(message) => {
            eprintln!("insight-simulator: {message}");
            std::process::exit(1);
        }
    };
    if let Err(err) = run(&config, &http_client).await {
        eprintln!("insight-simulator: {err}");
        std::process::exit(1);
    }
}

async fn run(config: &SimConfig, http_client: &reqwest::Client) -> Result<(), String> {
    match config.role {
        config::Role::Gateway => run_gateway(config, http_client).await,
        config::Role::Agentd => run_agentd(config, http_client).await,
    }
}

async fn run_gateway(config: &SimConfig, http_client: &reqwest::Client) -> Result<(), String> {
    if config.fetch_config {
        match gateway::fetch_initial_config(http_client, config).await {
            Ok(()) => {}
            Err(err) => eprintln!("warn initial_config_fetch failed: {err}"),
        }
    }
    loop {
        if let Err(err) = gateway::report_gateway_status(http_client, config).await {
            eprintln!("gateway status report failed: {err}");
        }
        if config.interval_secs == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(config.interval_secs)).await;
    }
    Ok(())
}

async fn run_agentd(config: &SimConfig, http_client: &reqwest::Client) -> Result<(), String> {
    let mut sequence: u64 = 0;
    loop {
        if let Err(err) = agentd::report_agent_status(http_client, config).await {
            eprintln!("agentd status report failed: {err}");
        }
        if config.report_action {
            if let Err(err) = agentd::report_action_result(http_client, config, sequence).await {
                eprintln!("agentd action result report failed: {err}");
            }
            sequence = sequence.wrapping_add(1);
        }
        if config.interval_secs == 0 {
            break;
        }
        tokio::time::sleep(Duration::from_secs(config.interval_secs)).await;
    }
    Ok(())
}

use std::time::Duration;

use insight_simulator::{agentd, client, config, config::SimConfig, gateway};

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mut config = match SimConfig::parse(&args) {
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
    if let Err(err) = run(&mut config, &http_client).await {
        eprintln!("insight-simulator: {err}");
        std::process::exit(1);
    }
}

async fn run(config: &mut SimConfig, http_client: &reqwest::Client) -> Result<(), String> {
    match config.role {
        config::Role::Gateway => run_gateway(config, http_client).await,
        config::Role::Agentd => run_agentd(config, http_client).await,
    }
}

async fn run_gateway(config: &mut SimConfig, http_client: &reqwest::Client) -> Result<(), String> {
    // 完整 onboarding：拉 initial-config（置备派生 RegistToken）→ register（换 RUNTIME_TOKEN）→
    // 之后用运行期凭据上报（401 防护）。
    if config.fetch_config {
        match gateway::fetch_initial_config(http_client, config).await {
            Ok((Some(regist_token), config_text)) => {
                match gateway::register(http_client, config, &regist_token).await {
                    Ok(bearer) => {
                        // 程序运行时生成网关配置：config.toml + runtime-token 落盘到
                        // <run_dir>/gateways/<gw>/<instance>/（先落盘再使用 bearer）。
                        if let Some(run_dir) = &config.run_dir {
                            let dir = std::path::Path::new(run_dir)
                                .join("gateways")
                                .join(&config.id)
                                .join(&config.instance_id);
                            if std::fs::create_dir_all(&dir).is_ok() {
                                let _ = std::fs::write(dir.join("config.toml"), &config_text);
                                let _ = std::fs::write(dir.join("runtime-token"), &bearer);
                                println!("event=ConfigWritten dir={}", dir.display());
                            }
                        }
                        config.token = bearer;
                        println!("event=Onboarded using runtime credential");
                    }
                    Err(err) => eprintln!("warn gateway register failed: {err}"),
                }
            }
            Ok((None, _)) => eprintln!("warn initial config fetched but no regist token"),
            Err(err) => eprintln!("warn initial_config_fetch failed: {err}"),
        }
    }
    loop {
        if let Err(err) = gateway::report_gateway_status(http_client, config).await {
            eprintln!("gateway status report failed: {err}");
        }
        if config.report_agents {
            if let Err(err) = gateway::report_agents_status(http_client, config).await {
                eprintln!("agent status report failed: {err}");
            }
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

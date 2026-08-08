// WarpInsightCenter 运行配置（env 驱动）。

use std::{env, error, fmt, path::PathBuf};

use crate::infra::sha256_hex;

const DEFAULT_LISTEN: &str = "127.0.0.1:3100";
const DEFAULT_STORE_PATH: &str = "state/warp-insight-center-store.json";
const ENV_LISTEN: &str = "WARP_INSIGHT_CENTER_LISTEN";
const ENV_STORE_PATH: &str = "WARP_INSIGHT_CENTER_STORE_PATH";
const ENV_GATEWAY_CREDENTIALS: &str = "WARP_INSIGHT_CENTER_GATEWAY_CREDENTIALS";
const ENV_ADMIN_TOKEN: &str = "WARP_INSIGHT_CENTER_ADMIN_TOKEN";

#[derive(Debug, Clone)]
pub struct CenterConfig {
    pub listen_addr: String,
    pub store_path: PathBuf,
    /// seed 网关凭证：gateway_id:token 列表，启动时写入 store（缺失才写）。
    pub gateway_credentials: Vec<GatewayCredentialSeed>,
    /// 管理面 token hash（读取接口迭代用，本次未启用）。
    pub admin_token_hash: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GatewayCredentialSeed {
    pub gateway_id: String,
    pub token: String,
    pub expires_at: Option<String>,
}

impl CenterConfig {
    pub fn load_from_env() -> Result<Self, ConfigError> {
        let listen_addr =
            env::var(ENV_LISTEN).unwrap_or_else(|_| DEFAULT_LISTEN.to_string());
        let store_path = PathBuf::from(
            env::var(ENV_STORE_PATH).unwrap_or_else(|_| DEFAULT_STORE_PATH.to_string()),
        );
        let gateway_credentials = parse_gateway_credentials(
            &env::var(ENV_GATEWAY_CREDENTIALS).unwrap_or_default(),
        )?;
        let admin_token = env::var(ENV_ADMIN_TOKEN)
            .ok()
            .filter(|value| !value.trim().is_empty());
        let admin_token_hash = admin_token.as_deref().map(sha256_hex);
        if listen_addr.trim().is_empty() {
            return Err(ConfigError::new("listen addr must not be empty"));
        }
        Ok(Self {
            listen_addr,
            store_path,
            gateway_credentials,
            admin_token_hash,
        })
    }
}

/// 解析 `gateway_id:token,gateway_id:token,...`。
fn parse_gateway_credentials(raw: &str) -> Result<Vec<GatewayCredentialSeed>, ConfigError> {
    let mut seeds = Vec::new();
    for entry in raw.split(',').map(str::trim).filter(|entry| !entry.is_empty()) {
        let Some((gateway_id, token)) = entry.split_once(':') else {
            return Err(ConfigError::new(format!(
                "invalid gateway credential entry {entry:?}: expected gateway_id:token"
            )));
        };
        let gateway_id = gateway_id.trim();
        let token = token.trim();
        if gateway_id.is_empty() || token.is_empty() {
            return Err(ConfigError::new(format!(
                "invalid gateway credential entry {entry:?}: gateway_id and token must not be empty"
            )));
        }
        seeds.push(GatewayCredentialSeed {
            gateway_id: gateway_id.to_string(),
            token: token.to_string(),
            expires_at: None,
        });
    }
    Ok(seeds)
}

#[derive(Debug, Clone)]
pub struct ConfigError(String);

impl ConfigError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl error::Error for ConfigError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_gateway_credential_seeds() {
        let seeds = parse_gateway_credentials("gw-001:tok-a,gw-002:tok-b").expect("seeds");
        assert_eq!(seeds.len(), 2);
        assert_eq!(seeds[0].gateway_id, "gw-001");
        assert_eq!(seeds[0].token, "tok-a");
        assert_eq!(seeds[1].gateway_id, "gw-002");
        assert_eq!(seeds[1].token, "tok-b");
    }

    #[test]
    fn rejects_malformed_seed_entry() {
        assert!(parse_gateway_credentials("gw-001").is_err());
        assert!(parse_gateway_credentials(":tok").is_err());
        assert!(parse_gateway_credentials("gw-001:").is_err());
    }

    #[test]
    fn empty_credentials_are_fine() {
        assert!(parse_gateway_credentials("").expect("empty").is_empty());
        assert!(parse_gateway_credentials(" , ").expect("blank").is_empty());
    }
}

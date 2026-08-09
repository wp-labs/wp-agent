// WarpInsightCenter 运行配置（env 驱动）。

use std::{env, error, fmt, path::PathBuf};

use crate::infra::sha256_hex;

const DEFAULT_LISTEN: &str = "127.0.0.1:3100";
const DEFAULT_STORE_PATH: &str = "state/warp-insight-center-store.json";
const ENV_LISTEN: &str = "WARP_INSIGHT_CENTER_LISTEN";
const ENV_STORE_PATH: &str = "WARP_INSIGHT_CENTER_STORE_PATH";
const ENV_GATEWAY_CREDENTIALS: &str = "WARP_INSIGHT_CENTER_GATEWAY_CREDENTIALS";
const ENV_ADMIN_TOKEN: &str = "WARP_INSIGHT_CENTER_ADMIN_TOKEN";
const ENV_DATABASE_URL: &str = "WARP_INSIGHT_CENTER_DATABASE_URL";
const ENV_VICTORIAMETRICS_URL: &str = "WARP_INSIGHT_CENTER_VICTORIAMETRICS_URL";
const ENV_PUBLIC_URL: &str = "WARP_INSIGHT_CENTER_PUBLIC_URL";
const ENV_GATEWAY_IMAGE: &str = "WARP_INSIGHT_CENTER_GATEWAY_IMAGE";
const ENV_ARTIFACT_DIR: &str = "WARP_INSIGHT_CENTER_ARTIFACT_DIR";
const ENV_OBJECT_STORAGE_ENDPOINT: &str = "WARP_INSIGHT_CENTER_OBJECT_STORAGE_ENDPOINT";
const ENV_OBJECT_STORAGE_BUCKET: &str = "WARP_INSIGHT_CENTER_OBJECT_STORAGE_BUCKET";
const ENV_OBJECT_STORAGE_ACCESS_KEY: &str = "WARP_INSIGHT_CENTER_OBJECT_STORAGE_ACCESS_KEY";
const ENV_OBJECT_STORAGE_SECRET_KEY: &str = "WARP_INSIGHT_CENTER_OBJECT_STORAGE_SECRET_KEY";

/// 默认对外地址使用域名（初始化 URL / 控制中心端点需要可被 Gateway 从外网访问）。
const DEFAULT_PUBLIC_URL: &str = "https://center.warpinsight.example";
const DEFAULT_GATEWAY_IMAGE: &str = "warp-gateway:latest";
const DEFAULT_ARTIFACT_DIR: &str = "artifacts";

#[derive(Debug, Clone)]
pub struct CenterConfig {
    pub listen_addr: String,
    pub store_path: PathBuf,
    /// seed 网关凭证：gateway_id:token 列表，启动时写入 store（缺失才写）。
    pub gateway_credentials: Vec<GatewayCredentialSeed>,
    /// 管理面 token hash（读取接口迭代用，本次未启用）。
    pub admin_token_hash: Option<String>,
    /// 开发期 PG 连接串：有值 → PgStore；未设置/为空 → FileStore。
    /// 推荐值即 compose 的 `postgres://demo:demo@127.0.0.1:55432/insight_demo`。
    pub database_url: Option<String>,
    /// 时序历史 VictoriaMetrics 基地址（如 compose 的 `http://127.0.0.1:8428`）。
    /// 有值 → 每次状态上报额外推送指标；未设置/为空 → 不启用（仅存快照）。
    pub victoriametrics_url: Option<String>,
    /// center 对外地址（生成网关初始化 URL），默认 `https://center.warpinsight.example`，
    /// 部署时通过 `WARP_INSIGHT_CENTER_PUBLIC_URL` 配置真实域名。
    pub public_url: String,
    /// gateway 镜像名（docker 安装命令 / 云镜像地址），默认 `warp-gateway:latest`。
    pub gateway_image: String,
    /// 本地制品目录（版本发布镜像制品落盘），默认 `artifacts/`。
    pub artifact_dir: PathBuf,
    /// 云对象存储（可选，S3 兼容 / MinIO）；配置了 endpoint 则发布制品存对象存储，否则本地。
    pub object_storage: Option<ObjectStorageConfig>,
}

/// S3 兼容对象存储配置（MinIO 等）。
#[derive(Debug, Clone)]
pub struct ObjectStorageConfig {
    pub endpoint: String,
    pub bucket: String,
    pub access_key: String,
    pub secret_key: String,
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
        // 有值 → PgStore；未设置或空串 → 回退 FileStore（无 PG 开发 / cargo test 可用）。
        let database_url = parse_optional_env_url(ENV_DATABASE_URL);
        // 有值 → 状态上报推送 VictoriaMetrics；未设置/空 → 不启用时序推送。
        let victoriametrics_url = parse_optional_env_url(ENV_VICTORIAMETRICS_URL);
        // 有值 → 用配置的对外地址/镜像；空 → 默认值。
        let public_url = env::var(ENV_PUBLIC_URL)
            .unwrap_or_else(|_| DEFAULT_PUBLIC_URL.to_string());
        let gateway_image = env::var(ENV_GATEWAY_IMAGE)
            .unwrap_or_else(|_| DEFAULT_GATEWAY_IMAGE.to_string());
        let artifact_dir = PathBuf::from(
            env::var(ENV_ARTIFACT_DIR).unwrap_or_else(|_| DEFAULT_ARTIFACT_DIR.to_string()),
        );
        // 对象存储：endpoint/bucket/凭据齐全才启用（否则用本地文件）。
        let object_storage = match (
            env::var(ENV_OBJECT_STORAGE_ENDPOINT).ok(),
            env::var(ENV_OBJECT_STORAGE_BUCKET).ok(),
            env::var(ENV_OBJECT_STORAGE_ACCESS_KEY).ok(),
            env::var(ENV_OBJECT_STORAGE_SECRET_KEY).ok(),
        ) {
            (Some(endpoint), Some(bucket), Some(access_key), Some(secret_key)) => {
                Some(ObjectStorageConfig {
                    endpoint,
                    bucket,
                    access_key,
                    secret_key,
                })
            }
            _ => None,
        };
        if listen_addr.trim().is_empty() {
            return Err(ConfigError::new("listen addr must not be empty"));
        }
        Ok(Self {
            listen_addr,
            store_path,
            gateway_credentials,
            admin_token_hash,
            database_url,
            victoriametrics_url,
            public_url,
            gateway_image,
            artifact_dir,
            object_storage,
        })
    }
}

/// 读取可选 env URL：未设置或空/纯空白 → None。
fn parse_optional_env_url(key: &str) -> Option<String> {
    env::var(key)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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

    #[test]
    fn parses_optional_env_url_trimming_blank() {
        // 测试专用 env key，避免与其他并行测试互相污染。
        let key = "WARP_INSIGHT_CENTER_TEST_OPTIONAL_URL";
        std::env::set_var(key, "http://127.0.0.1:8428");
        assert_eq!(
            parse_optional_env_url(key).as_deref(),
            Some("http://127.0.0.1:8428")
        );
        std::env::set_var(key, "  ");
        assert_eq!(parse_optional_env_url(key), None);
        std::env::remove_var(key);
        assert_eq!(parse_optional_env_url(key), None);
    }
}

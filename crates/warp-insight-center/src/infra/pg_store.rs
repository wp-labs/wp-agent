// WarpInsightCenter 状态存储：PostgreSQL 实现（开发期）。
// 连接 compose 提供的 postgres:16（127.0.0.1:55432，demo/demo，库 insight_demo），
// schema 由 docker/initdb/01_schema.sql 在首次启动时建表。

use insight_control::types::DateTime;
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};

use crate::config::GatewayCredentialSeed;

use super::{
    sha256_hex, GatewayStatusUpdate, StoredAgent, Store, StoreError, StoredGateway,
    StoredGatewayCredentialStatus,
};

#[derive(Debug, Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    pub async fn connect(database_url: &str) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(database_url)
            .await?;
        Ok(Self { pool })
    }
}

/// gateways 表行（与 docker/initdb/01_schema.sql 列一一对应）。
/// 时间列直接读为 chrono（sqlx postgres+chrono 特性映射 TIMESTAMPTZ）。
#[derive(Debug, FromRow)]
struct GatewayRow {
    gateway_id: String,
    instance_id: String,
    credential_token_hash: String,
    credential_status: String,
    credential_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    version: Option<String>,
    status: Option<String>,
    health: Option<String>,
    last_seen_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl GatewayRow {
    fn into_stored(self) -> StoredGateway {
        StoredGateway {
            gateway_id: self.gateway_id,
            instance_id: self.instance_id,
            credential_token_hash: self.credential_token_hash,
            credential_status: parse_credential_status(&self.credential_status),
            credential_expires_at: self.credential_expires_at.map(|value| value.to_rfc3339()),
            version: self.version,
            status: self.status,
            health: self.health,
            last_seen_at: self
                .last_seen_at
                .map(|value| {
                    DateTime::from_rfc3339(&value.to_rfc3339()).unwrap_or_else(DateTime::now)
                }),
        }
    }
}

/// agent_status 表行（与 docker/initdb/01_schema.sql 列对应）。
#[derive(Debug, FromRow)]
struct AgentRow {
    agent_id: String,
    gateway_id: String,
    instance_id: String,
    version: String,
    status: String,
    health: String,
    last_seen_at: chrono::DateTime<chrono::Utc>,
}

impl AgentRow {
    fn into_stored(self) -> StoredAgent {
        StoredAgent {
            agent_id: self.agent_id,
            gateway_id: self.gateway_id,
            instance_id: self.instance_id,
            version: self.version,
            status: self.status,
            health: self.health,
            last_seen_at: DateTime::from_rfc3339(&self.last_seen_at.to_rfc3339())
                .unwrap_or_else(DateTime::now),
        }
    }
}

/// DB 里 TEXT 存的枚举名 → 域枚举（未知值按 active 兜底，保持前向兼容）。
fn parse_credential_status(value: &str) -> StoredGatewayCredentialStatus {
    match value {
        "expired" => StoredGatewayCredentialStatus::Expired,
        "revoked" => StoredGatewayCredentialStatus::Revoked,
        _ => StoredGatewayCredentialStatus::Active,
    }
}

fn parse_optional_timestamptz(value: &Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    value
        .as_deref()
        .and_then(|text| chrono::DateTime::parse_from_rfc3339(text).ok())
        .map(|value| value.with_timezone(&chrono::Utc))
}

const GATEWAY_COLUMNS: &str = "gateway_id, instance_id, credential_token_hash, \
                               credential_status, credential_expires_at, \
                               version, status, health, last_seen_at";

#[async_trait::async_trait]
impl Store for PgStore {
    async fn seed(&self, seeds: &[GatewayCredentialSeed]) -> Result<bool, StoreError> {
        let mut added = false;
        for seed in seeds {
            let token_hash = sha256_hex(&seed.token);
            let expires_at = parse_optional_timestamptz(&seed.expires_at);
            let result = sqlx::query(
                "INSERT INTO gateways \
                    (gateway_id, instance_id, credential_token_hash, credential_expires_at) \
                 VALUES ($1, '', $2, $3) \
                 ON CONFLICT (gateway_id) DO NOTHING",
            )
            .bind(&seed.gateway_id)
            .bind(token_hash)
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
            added |= result.rows_affected() > 0;
        }
        Ok(added)
    }

    async fn list_gateways(&self) -> Result<Vec<StoredGateway>, StoreError> {
        let rows: Vec<GatewayRow> =
            sqlx::query_as(&format!("SELECT {GATEWAY_COLUMNS} FROM gateways ORDER BY gateway_id"))
                .fetch_all(&self.pool)
                .await?;
        Ok(rows.into_iter().map(GatewayRow::into_stored).collect())
    }

    async fn get_gateway(&self, gateway_id: &str) -> Result<Option<StoredGateway>, StoreError> {
        let row: Option<GatewayRow> = sqlx::query_as(&format!(
            "SELECT {GATEWAY_COLUMNS} FROM gateways WHERE gateway_id = $1"
        ))
        .bind(gateway_id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(GatewayRow::into_stored))
    }

    async fn upsert_gateway_status(&self, update: &GatewayStatusUpdate) -> Result<(), StoreError> {
        let last_seen_at = update.last_seen_at.to_chrono();
        sqlx::query(
            "UPDATE gateways \
             SET instance_id = $2, version = $3, status = $4, health = $5, last_seen_at = $6 \
             WHERE gateway_id = $1",
        )
        .bind(&update.gateway_id)
        .bind(&update.instance_id)
        .bind(&update.version)
        .bind(&update.status)
        .bind(&update.health)
        .bind(last_seen_at)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn create_gateway(
        &self,
        gateway_id: &str,
        token: &str,
    ) -> Result<StoredGateway, StoreError> {
        let token_hash = if token.is_empty() {
            String::new()
        } else {
            sha256_hex(token)
        };
        let result = sqlx::query(
            "INSERT INTO gateways (gateway_id, instance_id, credential_token_hash) \
             VALUES ($1, '', $2) \
             ON CONFLICT (gateway_id) DO NOTHING",
        )
        .bind(gateway_id)
        .bind(&token_hash)
        .execute(&self.pool)
        .await?;
        if result.rows_affected() == 0 {
            return Err(StoreError::Conflict(gateway_id.to_string()));
        }
        Ok(StoredGateway::provisioned(
            gateway_id.to_string(),
            String::new(),
            token_hash,
            None,
        ))
    }

    async fn upsert_agent_status(
        &self,
        gateway_id: &str,
        agents: &[StoredAgent],
    ) -> Result<(), StoreError> {
        for agent in agents {
            let last_seen_at = agent.last_seen_at.to_chrono();
            sqlx::query(
                "INSERT INTO agent_status \
                    (agent_id, gateway_id, instance_id, version, status, health, last_seen_at) \
                 VALUES ($1, $2, $3, $4, $5, $6, $7) \
                 ON CONFLICT (agent_id) DO UPDATE SET \
                   gateway_id = EXCLUDED.gateway_id, instance_id = EXCLUDED.instance_id, \
                   version = EXCLUDED.version, status = EXCLUDED.status, \
                   health = EXCLUDED.health, last_seen_at = EXCLUDED.last_seen_at",
            )
            .bind(&agent.agent_id)
            .bind(gateway_id)
            .bind(&agent.instance_id)
            .bind(&agent.version)
            .bind(&agent.status)
            .bind(&agent.health)
            .bind(last_seen_at)
            .execute(&self.pool)
            .await?;
        }
        Ok(())
    }

    async fn list_agents_by_gateway(
        &self,
        gateway_id: &str,
    ) -> Result<Vec<StoredAgent>, StoreError> {
        let rows: Vec<AgentRow> = sqlx::query_as(
            "SELECT agent_id, gateway_id, instance_id, version, status, health, last_seen_at \
             FROM agent_status WHERE gateway_id = $1 ORDER BY agent_id",
        )
        .bind(gateway_id)
        .fetch_all(&self.pool)
        .await?;
        Ok(rows.into_iter().map(AgentRow::into_stored).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_credential_status_with_unknown_fallback() {
        assert_eq!(
            parse_credential_status("active"),
            StoredGatewayCredentialStatus::Active
        );
        assert_eq!(
            parse_credential_status("expired"),
            StoredGatewayCredentialStatus::Expired
        );
        assert_eq!(
            parse_credential_status("revoked"),
            StoredGatewayCredentialStatus::Revoked
        );
        // 未知状态前向兼容：按 active 兜底。
        assert_eq!(
            parse_credential_status("suspended"),
            StoredGatewayCredentialStatus::Active
        );
    }

    #[test]
    fn parses_optional_timestamptz_from_rfc3339() {
        let value = Some("2026-08-08T12:00:00Z".to_string());
        let parsed = parse_optional_timestamptz(&value).expect("parse");
        assert_eq!(parsed.to_rfc3339(), "2026-08-08T12:00:00+00:00");
        assert_eq!(parse_optional_timestamptz(&None), None);
        assert_eq!(parse_optional_timestamptz(&Some("not-a-date".to_string())), None);
    }

    /// 集成测试：需本地 PostgreSQL（docker compose up -d postgres）。
    /// 用独立 gateway_id 避免污染，结束清理。默认连 compose 的 insight_demo 库，
    /// 可用 WARP_INSIGHT_CENTER_DATABASE_URL 覆盖。
    ///
    /// 运行：`cargo test -p warp-insight-center -- --ignored pg_store_round_trip`
    #[tokio::test]
    #[ignore = "需要本地 PostgreSQL（docker compose up -d postgres）"]
    async fn pg_store_round_trip() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let database_url = std::env::var("WARP_INSIGHT_CENTER_DATABASE_URL").unwrap_or_else(
            |_| "postgres://demo:demo@127.0.0.1:55432/insight_demo".to_string(),
        );
        let store = PgStore::connect(&database_url).await.expect("connect pg");

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let gateway_id = format!("pg-it-{nanos}");

        // seed：缺失时新增，重复 seed 不再新增。
        let seeds = vec![GatewayCredentialSeed {
            gateway_id: gateway_id.clone(),
            token: "it-secret".to_string(),
            expires_at: None,
        }];
        assert!(store.seed(&seeds).await.expect("seed"));
        assert!(!store.seed(&seeds).await.expect("seed again"));

        let stored = store
            .get_gateway(&gateway_id)
            .await
            .expect("get")
            .expect("seeded gateway");
        assert_eq!(stored.credential_token_hash, sha256_hex("it-secret"));
        assert_eq!(stored.credential_status, StoredGatewayCredentialStatus::Active);

        // 上报状态落库后读回（TIMESTAMPTZ ↔ chrono ↔ 共享 DateTime）。
        let reported_at = DateTime::now();
        store
            .upsert_gateway_status(&GatewayStatusUpdate {
                gateway_id: gateway_id.clone(),
                instance_id: "it-instance".to_string(),
                version: "v9.9.9".to_string(),
                status: "online".to_string(),
                health: "healthy".to_string(),
                last_seen_at: reported_at.clone(),
            })
            .await
            .expect("upsert status");

        let updated = store
            .get_gateway(&gateway_id)
            .await
            .expect("get updated")
            .expect("still present");
        assert_eq!(updated.instance_id, "it-instance");
        assert_eq!(updated.version.as_deref(), Some("v9.9.9"));
        assert_eq!(updated.status.as_deref(), Some("online"));
        assert_eq!(updated.health.as_deref(), Some("healthy"));
        assert_eq!(
            updated.last_seen_at.expect("last_seen_at").to_chrono(),
            reported_at.to_chrono()
        );

        // 列表包含该网关。
        let all = store.list_gateways().await.expect("list");
        assert!(all.iter().any(|gateway| gateway.gateway_id == gateway_id));

        // 清理测试数据。
        sqlx::query("DELETE FROM gateways WHERE gateway_id = $1")
            .bind(&gateway_id)
            .execute(&store.pool)
            .await
            .expect("cleanup");
    }

    /// 集成测试：create_gateway 的 PG 往返 + 重复创建冲突。需本地 PostgreSQL。
    #[tokio::test]
    #[ignore = "需要本地 PostgreSQL（docker compose up -d postgres）"]
    async fn pg_store_create_gateway_and_conflict() {
        use std::time::{SystemTime, UNIX_EPOCH};

        let database_url = std::env::var("WARP_INSIGHT_CENTER_DATABASE_URL").unwrap_or_else(
            |_| "postgres://demo:demo@127.0.0.1:55432/insight_demo".to_string(),
        );
        let store = PgStore::connect(&database_url).await.expect("connect pg");

        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        let gateway_id = format!("pg-create-{nanos}");

        let stored = store
            .create_gateway(&gateway_id, "it-token")
            .await
            .expect("create");
        assert_eq!(stored.gateway_id, gateway_id);
        assert_eq!(stored.credential_token_hash, sha256_hex("it-token"));
        assert_eq!(stored.credential_status, StoredGatewayCredentialStatus::Active);

        // 重复创建同一 gateway_id → Conflict。
        let err = store
            .create_gateway(&gateway_id, "other-token")
            .await
            .expect_err("conflict");
        assert!(matches!(err, StoreError::Conflict(ref gid) if gid == &gateway_id));

        // 清理测试数据。
        sqlx::query("DELETE FROM gateways WHERE gateway_id = $1")
            .bind(&gateway_id)
            .execute(&store.pool)
            .await
            .expect("cleanup");
    }
}

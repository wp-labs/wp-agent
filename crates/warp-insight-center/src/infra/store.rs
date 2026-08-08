// WarpInsightCenter 状态存储：Store trait + 双实现。
// - FileStore：JSON 文件快照（镜像 warp-gateway AdminStore），测试/无 PG 回退路径。
// - PgStore：PostgreSQL（开发期，见 pg_store.rs）。
// 持有每网关的凭证（sha256 hash）与最新上报状态（version/status/health/last_seen_at），
// 后续 GatewayStatusView / GatewayListView 从快照聚合读取。

use std::{
    collections::HashMap,
    error, fmt, fs,
    fs::OpenOptions,
    io,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, MutexGuard, OnceLock, Weak},
};

use insight_control::types::DateTime;
use serde::{Deserialize, Serialize};

use super::sha256_hex;
use crate::config::GatewayCredentialSeed;

#[derive(Debug, Clone)]
pub struct FileStore {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Json(serde_json::Error),
    Sql(sqlx::Error),
    /// 网关已存在（create_gateway 幂等冲突）。
    Conflict(String),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "center store io error: {err}"),
            Self::Json(err) => write!(f, "center store json error: {err}"),
            Self::Sql(err) => write!(f, "center store sql error: {err}"),
            Self::Conflict(gateway_id) => write!(f, "gateway {gateway_id} already exists"),
        }
    }
}

impl error::Error for StoreError {}

impl From<io::Error> for StoreError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(value: serde_json::Error) -> Self {
        Self::Json(value)
    }
}

impl From<sqlx::Error> for StoreError {
    fn from(value: sqlx::Error) -> Self {
        Self::Sql(value)
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CenterStoreSnapshot {
    /// key = gateway_id。
    pub gateways: HashMap<String, StoredGateway>,
    /// key = agent_id（Gateway 上报的其下 Agent 状态快照）。
    pub agents: HashMap<String, StoredAgent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGateway {
    pub gateway_id: String,
    pub instance_id: String,
    pub credential_token_hash: String,
    pub credential_status: StoredGatewayCredentialStatus,
    #[serde(default)]
    pub credential_expires_at: Option<String>,
    /// 最新上报状态（喂 GatewayStatusView，last_seen_at = reported_at）。
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub health: Option<String>,
    #[serde(default)]
    pub last_seen_at: Option<DateTime>,
}

impl StoredGateway {
    pub fn provisioned(
        gateway_id: String,
        instance_id: String,
        credential_token_hash: String,
        expires_at: Option<String>,
    ) -> Self {
        Self {
            gateway_id,
            instance_id,
            credential_token_hash,
            credential_status: StoredGatewayCredentialStatus::Active,
            credential_expires_at: expires_at,
            version: None,
            status: None,
            health: None,
            last_seen_at: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoredGatewayCredentialStatus {
    Active,
    Expired,
    Revoked,
}

/// Gateway 上报的其下 Agent 状态快照。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredAgent {
    pub agent_id: String,
    pub gateway_id: String,
    pub instance_id: String,
    pub version: String,
    pub status: String,
    pub health: String,
    pub last_seen_at: DateTime,
}

/// 单次状态上报落库参数（ReceiveGatewayStatusReport → store）。
#[derive(Debug, Clone)]
pub struct GatewayStatusUpdate {
    pub gateway_id: String,
    pub instance_id: String,
    pub version: String,
    pub status: String,
    pub health: String,
    pub last_seen_at: DateTime,
}

/// 网关凭证与状态的持久化抽象：FileStore（JSON 文件，测试/无 PG 回退）与
/// PgStore（PostgreSQL，开发期）双实现。
/// 原生 async fn in trait 在当前工具链下不可 dyn（E0038），故用 `#[async_trait]`
/// （内部转为 `Pin<Box<dyn Future + Send>>`，trait 保持 `Send + Sync` 可 `Arc<dyn Store>`）。
#[async_trait::async_trait]
pub trait Store: Send + Sync + std::fmt::Debug {
    /// 用 seed 凭证补齐缺失的网关（已有条目不覆盖，保留已轮换凭证）。返回是否有新增。
    async fn seed(&self, seeds: &[GatewayCredentialSeed]) -> Result<bool, StoreError>;
    /// 全部网关（含已接入未上报的），按 gateway_id 排序。
    async fn list_gateways(&self) -> Result<Vec<StoredGateway>, StoreError>;
    async fn get_gateway(&self, gateway_id: &str) -> Result<Option<StoredGateway>, StoreError>;
    /// 落库最新上报状态（仅更新已接入网关，与 FileStore 的 update 语义一致）。
    async fn upsert_gateway_status(&self, update: &GatewayStatusUpdate) -> Result<(), StoreError>;
    /// 创建网关实例：gateway_id 已存在 → `Err(Conflict)`；否则以空 instance_id 接入。
    /// token 非空则存 `sha256(token)`，空 token → 空 hash（无凭证网关无法上报）。
    async fn create_gateway(
        &self,
        gateway_id: &str,
        token: &str,
    ) -> Result<StoredGateway, StoreError>;
    /// 落库 Gateway 上报的其下 Agent 状态（按 agent_id 幂等 upsert，记录归属 gateway_id）。
    async fn upsert_agent_status(
        &self,
        gateway_id: &str,
        agents: &[StoredAgent],
    ) -> Result<(), StoreError>;
    /// 查询某 gateway 下的全部 Agent 状态（按 agent_id 排序）。
    async fn list_agents_by_gateway(
        &self,
        gateway_id: &str,
    ) -> Result<Vec<StoredAgent>, StoreError>;
}

impl FileStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let lock = shared_process_lock(&path);
        Self { path, lock }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn load(&self) -> Result<CenterStoreSnapshot, StoreError> {
        let _guard = self.lock()?;
        self.load_snapshot()
    }

    pub fn save(&self, snapshot: &CenterStoreSnapshot) -> Result<(), StoreError> {
        let _guard = self.lock()?;
        self.save_snapshot(snapshot)
    }

    pub fn update<T>(
        &self,
        mutate: impl FnOnce(&mut CenterStoreSnapshot) -> T,
    ) -> Result<T, StoreError> {
        let _guard = self.lock()?;
        let mut snapshot = self.load_snapshot()?;
        let output = mutate(&mut snapshot);
        self.save_snapshot(&snapshot)?;
        Ok(output)
    }

    /// 用 seed 凭证补齐缺失的网关（已有条目不覆盖，保留已轮换凭证）。返回是否有新增。
    pub fn seed(&self, seeds: &[GatewayCredentialSeed]) -> Result<bool, StoreError> {
        let seeds = seeds.to_vec();
        self.update(|snapshot| {
            let mut added = false;
            for seed in &seeds {
                if snapshot.gateways.contains_key(&seed.gateway_id) {
                    continue;
                }
                snapshot.gateways.insert(
                    seed.gateway_id.clone(),
                    StoredGateway::provisioned(
                        seed.gateway_id.clone(),
                        String::new(),
                        sha256_hex(&seed.token),
                        seed.expires_at.clone(),
                    ),
                );
                added = true;
            }
            added
        })
    }

    /// 创建网关实例（同步，供测试与 trait 委托）：gateway_id 已存在 → Conflict。
    pub fn create_gateway(
        &self,
        gateway_id: &str,
        token: &str,
    ) -> Result<StoredGateway, StoreError> {
        self.update(|snapshot| {
            if snapshot.gateways.contains_key(gateway_id) {
                return Err(StoreError::Conflict(gateway_id.to_string()));
            }
            let token_hash = if token.is_empty() {
                String::new()
            } else {
                sha256_hex(token)
            };
            let stored = StoredGateway::provisioned(
                gateway_id.to_string(),
                String::new(),
                token_hash,
                None,
            );
            snapshot.gateways.insert(gateway_id.to_string(), stored.clone());
            Ok(stored)
        })?
    }

    /// 落库 Agent 状态（同步，供测试与 trait 委托）：按 agent_id upsert，归属以 gateway_id 参数为准。
    pub fn upsert_agent_status(
        &self,
        gateway_id: &str,
        agents: &[StoredAgent],
    ) -> Result<(), StoreError> {
        self.update(|snapshot| {
            for mut agent in agents.to_vec() {
                agent.gateway_id = gateway_id.to_string();
                snapshot.agents.insert(agent.agent_id.clone(), agent);
            }
        })
    }

    pub fn list_agents_by_gateway(
        &self,
        gateway_id: &str,
    ) -> Result<Vec<StoredAgent>, StoreError> {
        let snapshot = self.load()?;
        let mut agents: Vec<StoredAgent> = snapshot
            .agents
            .into_values()
            .filter(|agent| agent.gateway_id == gateway_id)
            .collect();
        agents.sort_by(|left, right| left.agent_id.cmp(&right.agent_id));
        Ok(agents)
    }

    fn lock(&self) -> Result<StoreLockGuard<'_>, StoreError> {
        let process_guard = self
            .lock
            .lock()
            .map_err(|_| StoreError::Io(io::Error::other("center store lock poisoned")))?;
        let file_guard = FileLockGuard::lock(&lock_file_path(&self.path))?;
        Ok(StoreLockGuard {
            _process_guard: process_guard,
            _file_guard: file_guard,
        })
    }

    fn load_snapshot(&self) -> Result<CenterStoreSnapshot, StoreError> {
        if !self.path.exists() {
            return Ok(CenterStoreSnapshot::default());
        }
        let content = fs::read_to_string(&self.path)?;
        if content.trim().is_empty() {
            return Ok(CenterStoreSnapshot::default());
        }
        Ok(serde_json::from_str(&content)?)
    }

    fn save_snapshot(&self, snapshot: &CenterStoreSnapshot) -> Result<(), StoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(snapshot)?;
        let temp_path = temp_store_path(&self.path);
        let write_result = (|| -> Result<(), StoreError> {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = options.open(&temp_path)?;
            file.write_all(content.as_bytes())?;
            file.write_all(b"\n")?;
            file.sync_all()?;
            fs::rename(&temp_path, &self.path)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&self.path, fs::Permissions::from_mode(0o600))?;
            }
            if let Some(parent) = self.path.parent() {
                sync_directory(parent)?;
            }
            Ok(())
        })();
        if write_result.is_err() {
            let _ = fs::remove_file(&temp_path);
        }
        write_result?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl Store for FileStore {
    async fn seed(&self, seeds: &[GatewayCredentialSeed]) -> Result<bool, StoreError> {
        // 显式调用同步固有方法，避免与 trait 方法同名递归。
        FileStore::seed(self, seeds)
    }

    async fn list_gateways(&self) -> Result<Vec<StoredGateway>, StoreError> {
        let snapshot = self.load()?;
        let mut gateways: Vec<StoredGateway> = snapshot.gateways.into_values().collect();
        gateways.sort_by(|left, right| left.gateway_id.cmp(&right.gateway_id));
        Ok(gateways)
    }

    async fn get_gateway(&self, gateway_id: &str) -> Result<Option<StoredGateway>, StoreError> {
        let snapshot = self.load()?;
        Ok(snapshot.gateways.get(gateway_id).cloned())
    }

    async fn upsert_gateway_status(&self, update: &GatewayStatusUpdate) -> Result<(), StoreError> {
        self.update(|snapshot| {
            if let Some(stored) = snapshot.gateways.get_mut(&update.gateway_id) {
                stored.instance_id = update.instance_id.clone();
                stored.version = Some(update.version.clone());
                stored.status = Some(update.status.clone());
                stored.health = Some(update.health.clone());
                stored.last_seen_at = Some(update.last_seen_at.clone());
            }
        })
    }

    async fn create_gateway(
        &self,
        gateway_id: &str,
        token: &str,
    ) -> Result<StoredGateway, StoreError> {
        // 显式调用同步固有方法，避免与 trait 方法同名递归。
        FileStore::create_gateway(self, gateway_id, token)
    }

    async fn upsert_agent_status(
        &self,
        gateway_id: &str,
        agents: &[StoredAgent],
    ) -> Result<(), StoreError> {
        FileStore::upsert_agent_status(self, gateway_id, agents)
    }

    async fn list_agents_by_gateway(
        &self,
        gateway_id: &str,
    ) -> Result<Vec<StoredAgent>, StoreError> {
        FileStore::list_agents_by_gateway(self, gateway_id)
    }
}

struct StoreLockGuard<'a> {
    _process_guard: MutexGuard<'a, ()>,
    _file_guard: FileLockGuard,
}

static PROCESS_LOCKS: OnceLock<Mutex<HashMap<PathBuf, Weak<Mutex<()>>>>> = OnceLock::new();

fn shared_process_lock(path: &Path) -> Arc<Mutex<()>> {
    let key = normalized_store_path(path);
    let locks = PROCESS_LOCKS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut locks = locks.lock().expect("center store process lock map poisoned");
    if let Some(lock) = locks.get(&key).and_then(Weak::upgrade) {
        return lock;
    }
    let lock = Arc::new(Mutex::new(()));
    locks.insert(key, Arc::downgrade(&lock));
    lock
}

fn normalized_store_path(path: &Path) -> PathBuf {
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::env::current_dir()
        .map(|cwd| cwd.join(path))
        .unwrap_or_else(|_| path.to_path_buf())
}

fn lock_file_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("center-store.json");
    path.with_file_name(format!(".{file_name}.lock"))
}

fn temp_store_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("center-store.json");
    let nanos = chrono::Utc::now()
        .timestamp_nanos_opt()
        .unwrap_or_else(|| chrono::Utc::now().timestamp_micros() * 1_000);
    path.with_file_name(format!(".{file_name}.{}.{}.tmp", std::process::id(), nanos))
}

#[cfg(unix)]
struct FileLockGuard {
    file: fs::File,
}

#[cfg(unix)]
impl FileLockGuard {
    fn lock(path: &Path) -> Result<Self, StoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;
        lock_file_exclusive(&file)?;
        Ok(Self { file })
    }
}

#[cfg(unix)]
impl Drop for FileLockGuard {
    fn drop(&mut self) {
        let _ = unlock_file(&self.file);
    }
}

#[cfg(not(unix))]
struct FileLockGuard;

#[cfg(not(unix))]
impl FileLockGuard {
    fn lock(_path: &Path) -> Result<Self, StoreError> {
        Ok(Self)
    }
}

#[cfg(unix)]
fn lock_file_exclusive(file: &fs::File) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    const LOCK_EX: std::os::raw::c_int = 2;
    if unsafe { flock(file.as_raw_fd(), LOCK_EX) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
fn unlock_file(file: &fs::File) -> io::Result<()> {
    use std::os::fd::AsRawFd;
    const LOCK_UN: std::os::raw::c_int = 8;
    if unsafe { flock(file.as_raw_fd(), LOCK_UN) } == 0 {
        Ok(())
    } else {
        Err(io::Error::last_os_error())
    }
}

#[cfg(unix)]
extern "C" {
    fn flock(fd: std::os::raw::c_int, operation: std::os::raw::c_int) -> std::os::raw::c_int;
}

fn sync_directory(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        OpenOptions::new().read(true).open(path)?.sync_all()
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn seed_provisions_missing_gateways_only() {
        let path = test_store_path();
        let store = FileStore::new(&path);
        let seeds = vec![
            GatewayCredentialSeed {
                gateway_id: "gw-001".to_string(),
                token: "tok-a".to_string(),
                expires_at: None,
            },
            GatewayCredentialSeed {
                gateway_id: "gw-002".to_string(),
                token: "tok-b".to_string(),
                expires_at: None,
            },
        ];
        assert!(store.seed(&seeds).expect("seed"));
        assert!(store.seed(&seeds).expect("seed again") == false);

        let snapshot = store.load().expect("load");
        assert_eq!(snapshot.gateways.len(), 2);
        let first = &snapshot.gateways["gw-001"];
        assert_eq!(first.credential_token_hash, sha256_hex("tok-a"));
        assert_eq!(first.credential_status, StoredGatewayCredentialStatus::Active);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn update_upserts_latest_status() {
        let path = test_store_path();
        let store = FileStore::new(&path);
        let seeds = vec![GatewayCredentialSeed {
            gateway_id: "gw-001".to_string(),
            token: "tok-a".to_string(),
            expires_at: None,
        }];
        store.seed(&seeds).expect("seed");

        store
            .update(|snapshot| {
                if let Some(stored) = snapshot.gateways.get_mut("gw-001") {
                    stored.version = Some("v2.4.1".to_string());
                    stored.status = Some("online".to_string());
                    stored.health = Some("healthy".to_string());
                    stored.last_seen_at = Some(DateTime::now());
                }
            })
            .expect("update");

        let snapshot = store.load().expect("load");
        let stored = &snapshot.gateways["gw-001"];
        assert_eq!(stored.version.as_deref(), Some("v2.4.1"));
        assert_eq!(stored.status.as_deref(), Some("online"));
        assert!(stored.last_seen_at.is_some());
        let _ = fs::remove_file(path);
    }

    #[test]
    fn create_gateway_provisions_and_conflicts_on_duplicate() {
        let path = test_store_path();
        let store = FileStore::new(&path);

        let stored = store.create_gateway("gw-100", "tok-x").expect("create");
        assert_eq!(stored.gateway_id, "gw-100");
        assert_eq!(stored.credential_token_hash, sha256_hex("tok-x"));
        assert_eq!(stored.credential_status, StoredGatewayCredentialStatus::Active);
        assert_eq!(stored.instance_id, "");

        // 重复创建同一 gateway_id → Conflict。
        let err = store.create_gateway("gw-100", "tok-y").expect_err("conflict");
        assert!(matches!(err, StoreError::Conflict(ref gateway_id) if gateway_id == "gw-100"));

        // 空 token → 空 hash（无凭证网关无法上报）。
        let stored = store.create_gateway("gw-nocred", "").expect("create no credential");
        assert_eq!(stored.credential_token_hash, "");

        let _ = fs::remove_file(path);
    }

    fn test_store_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("warp-insight-center-store-test-{nanos}.json"))
    }
}

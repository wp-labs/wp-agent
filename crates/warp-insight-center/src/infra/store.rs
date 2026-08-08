// WarpInsightCenter 状态存储：JSON 文件快照（镜像 warp-gateway AdminStore）。
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
pub struct CenterStore {
    path: PathBuf,
    lock: Arc<Mutex<()>>,
}

#[derive(Debug)]
pub enum StoreError {
    Io(io::Error),
    Json(serde_json::Error),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(err) => write!(f, "center store io error: {err}"),
            Self::Json(err) => write!(f, "center store json error: {err}"),
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct CenterStoreSnapshot {
    /// key = gateway_id。
    pub gateways: HashMap<String, StoredGateway>,
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

impl CenterStore {
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
        let store = CenterStore::new(&path);
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
        let store = CenterStore::new(&path);
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

    fn test_store_path() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time")
            .as_nanos();
        std::env::temp_dir().join(format!("warp-insight-center-store-test-{nanos}.json"))
    }
}

// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.HostInventory")]
pub struct HostRuntimeState {
    pub host_id: String,
    pub observed_at: crate::DateTime,
    pub boot_id: Option<String>,
    pub uptime_seconds: Option<i64>,
    pub loadavg_1m: Option<f64>,
    pub loadavg_5m: Option<f64>,
    pub loadavg_15m: Option<f64>,
    pub cpu_usage_pct: Option<f64>,
    pub memory_used_bytes: Option<i64>,
    pub memory_available_bytes: Option<i64>,
    pub disk_used_bytes: Option<i64>,
    pub disk_available_bytes: Option<i64>,
    pub process_count: Option<i64>,
    pub container_count: Option<i64>,
    pub agent_health: crate::AgentHealth,
    pub protection_state: Option<crate::ProtectionState>,
    pub last_error: Option<String>,
}

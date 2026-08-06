// @moju generated
// @moju hash=a412e80413f808bc

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Reporting", module = "Reporting.Health")]
pub struct RuntimeHealthSnapshot {
    pub running_count: i64,
    pub state: crate::control::types::HealthState,
    pub queue_depth: i64,
    pub metrics: crate::control::types::MetricsHealthSnapshot,
    pub updated_at: crate::control::types::DateTime,
    pub reporting_count: i64,
    pub discovery: String,
}

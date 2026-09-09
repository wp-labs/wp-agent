// @jumo generated
// @jumo hash=a412e80413f808bc

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Health")]
pub struct RuntimeHealthSnapshot {
    pub running_count: i64,
    pub state: crate::HealthState,
    pub queue_depth: i64,
    pub metrics: crate::MetricsHealthSnapshot,
    pub updated_at: crate::DateTime,
    pub reporting_count: i64,
    pub discovery: String,
}

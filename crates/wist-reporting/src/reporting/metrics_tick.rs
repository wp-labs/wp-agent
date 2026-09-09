// @jumo generated
// @jumo hash=8b378da3c456e28d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Health")]
pub struct MetricsTick {
    pub failures: String,
    pub health_snapshot: String,
}

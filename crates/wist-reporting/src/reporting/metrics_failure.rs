// @jumo generated
// @jumo hash=475960500263201f

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Health")]
pub struct MetricsFailure {
    pub detail: String,
    pub kind: String,
}

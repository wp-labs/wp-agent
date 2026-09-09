// @jumo generated
// @jumo hash=0abf4c75304819d9

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Health")]
pub struct TelemetryFailure {
    pub kind: String,
    pub detail: String,
}

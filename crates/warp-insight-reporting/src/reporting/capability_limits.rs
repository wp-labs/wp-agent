// @jumo generated
// @jumo hash=60aa6a4fb865dc14

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct CapabilityLimits {
    pub max_duration_ms: String,
    pub max_stderr_bytes: String,
    pub max_stdout_bytes: String,
}

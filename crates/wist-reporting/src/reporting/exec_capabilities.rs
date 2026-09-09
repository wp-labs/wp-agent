// @jumo generated
// @jumo hash=47a438ed8b14e723

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct ExecCapabilities {
    pub cancel_supported: String,
    pub max_concurrent: String,
    pub supported: String,
}

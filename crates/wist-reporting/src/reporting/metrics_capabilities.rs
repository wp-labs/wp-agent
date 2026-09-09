// @jumo generated
// @jumo hash=605bb21e8630f7ba

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Contract")]
pub struct MetricsCapabilities {
    pub supported: String,
    pub collection_kinds: String,
}

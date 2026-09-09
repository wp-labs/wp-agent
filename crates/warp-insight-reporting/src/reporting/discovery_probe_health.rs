// @jumo generated
// @jumo hash=1b7d2ec658e8966e

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Reporting", module = "Reporting.Health")]
pub struct DiscoveryProbeHealth {
    pub status: String,
    pub resource_count: String,
    pub target_count: String,
    pub probe: String,
    pub source: String,
    pub phase: String,
    pub error: String,
}

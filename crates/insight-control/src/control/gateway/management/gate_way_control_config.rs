// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.Gateway.Management")]
pub struct GateWayControlConfig {
    #[moju(unique)]
    pub config_id: String,
    pub gateway_id: String,
    pub advertise_url: String,
    pub enrollment_url: String,
    pub gateway_url: String,
    pub telemetry_url: String,
    pub updated_at: crate::DateTime,
}

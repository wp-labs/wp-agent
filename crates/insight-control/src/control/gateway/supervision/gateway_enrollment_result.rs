// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GatewayEnrollmentResult {
    pub status: String,
    pub gateway_id: String,
    pub instance_id: String,
    pub credential_id: String,
    pub initial_config: String,
}

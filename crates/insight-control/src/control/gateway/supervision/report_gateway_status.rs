// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct ReportGatewayStatus {
    pub gateway_id: String,
    pub instance_id: String,
    pub version: String,
    pub status: String,
    pub health: String,
    pub reported_at: crate::DateTime,
}

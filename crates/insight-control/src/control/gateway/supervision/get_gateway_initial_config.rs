// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GetGatewayInitialConfig {
    pub instance_id: String,
    pub requested_at: crate::DateTime,
}

// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct VerifyGatewayCredential {
    pub gateway_id: String,
    pub instance_id: String,
    pub credential_id: String,
}

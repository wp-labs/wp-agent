// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Security")]
pub struct GatewayCredentialVerificationResult {
    pub gateway_id: String,
    pub credential_id: String,
    pub status: String,
    pub verified_at: crate::DateTime,
}

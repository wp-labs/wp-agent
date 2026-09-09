// @jumo generated
// Model struct: GatewayCredentialBundle — 网关注册成功后签发的运行期凭据（RUNTIME_TOKEN）。
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Security")]
pub struct GatewayCredentialBundle {
    #[jumo(unique)]
    pub credential_id: String,
    pub gateway_id: String,
    pub instance_id: String,
    pub auth_scheme: String,
    pub bearer_token: String,
    pub issued_at: crate::DateTime,
    pub expires_at: crate::DateTime,
}

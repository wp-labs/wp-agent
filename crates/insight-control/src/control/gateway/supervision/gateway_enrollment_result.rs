// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GatewayEnrollmentResult {
    pub status: String,
    pub gateway_id: String,
    pub instance_id: String,
    pub credential_id: String,
    pub initial_config: String,
    /// 注册成功后签发的独立运行期凭据（RUNTIME_TOKEN）：旧注册凭据（RegistToken）只用于
    /// 本次注册，运行期 Bearer 以本 bundle 为准。
    pub credential_bundle: crate::GatewayCredentialBundle,
}

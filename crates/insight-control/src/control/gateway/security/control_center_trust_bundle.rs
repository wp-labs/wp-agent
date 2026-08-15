// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.Gateway.Security")]
pub struct ControlCenterTrustBundle {
    pub trust_bundle_id: String,
    pub control_endpoint: String,
    /// control-center.pem 公钥内容——最终交给 gateway 校验中心 TLS。
    pub ca_bundle: String,
    pub server_name: String,
    pub expected_san: String,
    pub issued_at: Option<crate::DateTime>,
    pub expires_at: Option<crate::DateTime>,
}

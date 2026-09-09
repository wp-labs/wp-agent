// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Identity")]
pub struct GateWayIdentity {
    #[jumo(unique)]
    pub gateway_id: String,
    pub instance_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub node_id: String,
    pub issued_at: crate::DateTime,
    pub expires_at: crate::DateTime,
    pub status: crate::GateWayIdentityStatus,
}

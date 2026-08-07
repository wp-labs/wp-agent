// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GateWayIdentity")]
pub struct GateWayIdentity {
    #[moju(unique)]
    pub gateway_id: String,
    pub instance_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub node_id: String,
    pub issued_at: crate::DateTime,
    pub expires_at: crate::DateTime,
    pub status: crate::GateWayIdentityStatus,
}

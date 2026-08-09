// @moju generated
use super::GatewayInstanceLifecycleState;

#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GatewayInstance {
    pub gateway_id: String,
    pub instance_id: String,
    pub lifecycle_state: GatewayInstanceLifecycleState,
    pub created_at: crate::DateTime,
    pub initialized_at: Option<crate::DateTime>,
}

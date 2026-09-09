// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.GatewayApp.FacingInterface")]
pub struct GatewayInitializationStatus {
    pub gateway_id: String,
    pub instance_id: Option<String>,
    pub lifecycle_state: crate::GatewayInstanceLifecycleState,
    pub initialized: bool,
}

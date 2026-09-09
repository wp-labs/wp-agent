// @jumo generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Gateway.Supervision")]
pub enum GatewayInstanceLifecycleState {
    Provisioned,
    Initializing,
    Running,
    Failed,
}

// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "state", domain = "Control", module = "Control.Gateway.InstanceManagement")]
pub enum GatewayInstanceLifecycleState {
    Provisioned,
    Initializing,
    Running,
    Failed,
}

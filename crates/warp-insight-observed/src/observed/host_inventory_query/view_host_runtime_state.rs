// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Observed", module = "Observed.HostInventoryQuery")]
pub struct ViewHostRuntimeState {
    pub host_id: String,
    pub requested_by: String,
}

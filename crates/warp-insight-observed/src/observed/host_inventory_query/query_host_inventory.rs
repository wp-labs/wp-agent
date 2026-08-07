// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Observed", module = "Observed.HostInventoryQuery")]
pub struct QueryHostInventory {
    pub tenant_id: Option<String>,
    pub environment_id: Option<String>,
    pub host_name: Option<String>,
    pub requested_by: String,
}

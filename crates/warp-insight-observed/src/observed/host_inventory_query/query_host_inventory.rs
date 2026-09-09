// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Observed", module = "Observed.HostInventoryQuery")]
pub struct QueryHostInventory {
    pub tenant_id: Option<String>,
    pub environment_id: Option<String>,
    pub host_name: Option<String>,
    pub requested_by: String,
}

// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenterApp.AdminFacingInterface")]
pub struct AdminQueryHostInventory {
    pub tenant_id: Option<String>,
    pub environment_id: Option<String>,
    pub host_name: Option<String>,
    pub requested_by: String,
}

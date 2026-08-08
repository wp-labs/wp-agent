// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenterApp.AdminFacingInterface")]
pub struct AdminViewSoftwareVulnerabilities {
    pub software_id: Option<String>,
    pub host_id: Option<String>,
    pub requested_by: String,
}

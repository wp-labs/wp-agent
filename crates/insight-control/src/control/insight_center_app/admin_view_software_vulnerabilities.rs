// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Control", module = "Control.InsightCenterApp.AdminFacingInterface")]
pub struct AdminViewSoftwareVulnerabilities {
    pub software_id: Option<String>,
    pub host_id: Option<String>,
    pub requested_by: String,
}

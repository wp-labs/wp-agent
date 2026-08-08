// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "response", domain = "Control", module = "Control.InsightCenterApp.AdminFacingInterface")]
pub struct AdminSoftwareVulnerabilitiesReturned {
    pub findings: Vec<warp_insight_security::SoftwareVulnerabilityFinding>,
}

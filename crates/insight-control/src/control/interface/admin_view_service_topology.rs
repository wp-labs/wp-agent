// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "command", domain = "Control", module = "Control.AdminFacingInterface")]
pub struct AdminViewServiceTopology {
    pub business_id: Option<String>,
    pub service_id: Option<String>,
    pub requested_by: String,
}

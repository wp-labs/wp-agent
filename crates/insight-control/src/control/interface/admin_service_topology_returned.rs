// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "response", domain = "Control", module = "Control.AdminFacingInterface")]
pub struct AdminServiceTopologyReturned {
    pub services: Vec<warp_insight_observed::ServiceEntity>,
}

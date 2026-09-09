// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "response", domain = "Control", module = "Control.InsightCenterApp.AdminFacingInterface")]
pub struct AdminServiceTopologyReturned {
    pub services: Vec<wist_observed::ServiceEntity>,
}

// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "message", role = "response", domain = "Control", module = "Control.AdminFacingInterface")]
pub struct AdminEffectiveResponsibilityReturned {
    pub responsibility: warp_insight_observed::EffectiveHostResponsibility,
}

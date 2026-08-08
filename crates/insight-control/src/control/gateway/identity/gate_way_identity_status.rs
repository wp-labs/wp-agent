// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "state", domain = "Control", module = "Control.Gateway.Identity")]
pub enum GateWayIdentityStatus {
    Active,
    Revoked,
    Expired,
    RenewalRequired,
}

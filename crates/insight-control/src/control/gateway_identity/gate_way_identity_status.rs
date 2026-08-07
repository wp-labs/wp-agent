// @moju generated
#[derive(Debug, Clone, Copy, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "variant", domain = "Control", module = "Control.GateWayIdentity")]
pub enum GateWayIdentityStatus {
    Active,
    Revoked,
    Expired,
    RenewalRequired,
}

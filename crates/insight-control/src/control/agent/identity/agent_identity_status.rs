// @jumo generated
// @jumo hash=df19454bec80dade

#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Agent.Identity")]
pub enum AgentIdentityStatus {
    Active,
    Revoked,
    Expired,
    RenewalRequired,
}

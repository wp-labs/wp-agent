// @jumo generated
// @jumo hash=85bf9c3b74cb8b68

#[derive(Debug, Clone, PartialEq, Eq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "state", domain = "Control", module = "Control.Agent.Identity")]
pub enum AgentCredentialVerificationStatus {
    Verified,
    UnknownAgent,
    CredentialMissing,
    CredentialExpired,
    CredentialRevoked,
    CredentialMismatch,
}

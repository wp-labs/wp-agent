// @jumo generated
// @jumo hash=c3988499d829088d

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Identity")]
pub struct AgentControlAuthProfile {
    pub enrollment_token_rejected: bool,
    pub mtls_client_certificate_allowed: bool,
    pub agent_credential_required: bool,
    pub server_tls_required: bool,
}

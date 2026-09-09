// @jumo generated
// @jumo hash=b36d8961ec872e59

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentEnrollmentAuthProfile {
    pub trust_bundle_required: bool,
    pub enrollment_token_required: bool,
    pub credential_request_required: bool,
    pub server_tls_required: bool,
}

// @jumo generated
// @jumo hash=c75502bb722a32ca

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentEnrollmentResult {
    pub credential_bundle: crate::control::types::AgentCredentialBundle,
    pub initial_config: crate::control::types::AgentInitialConfig,
    pub reason_code: String,
    pub instance_id: String,
    pub agent_id: String,
    pub policy_binding: crate::control::types::AgentPolicyBinding,
    pub status: crate::control::types::AgentEnrollmentResultStatus,
    pub issued_identity: crate::control::types::AgentIdentity,
}

// @jumo generated
// @jumo hash=c04fc3172036cc4c

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentEnrollmentTokenValidation {
    pub host_profile: crate::control::types::AgentHostProfile,
    pub validated_at: crate::control::types::DateTime,
    pub status: crate::control::types::AgentEnrollmentTokenValidationStatus,
    pub tenant_id: String,
    pub environment_id: String,
    pub token_id: String,
    pub reason_code: String,
}

// @jumo generated
// @jumo hash=5a7809976f6c209f

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Agent.Enrollment")]
pub struct AgentBootstrapBundle {
    pub expires_at: crate::control::types::DateTime,
    pub environment_id: String,
    pub control_endpoint: String,
    pub tenant_id: String,
    #[jumo(unique)]
    pub bundle_id: String,
    pub install_script_url: String,
    pub agent_package_url: String,
    pub agent_package_sha256: String,
    pub trust_bundle: String,
}

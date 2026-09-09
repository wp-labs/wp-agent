// @jumo generated
// @jumo hash=ac6a7c4539bc6948

#[derive(Debug, Clone, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(
    kind = "struct",
    domain = "Control",
    module = "Control.Agent.Enrollment"
)]
pub struct AgentInstallCode {
    pub x86_linux_install_code: String,
    pub bootstrap_enrollment_token: String,
    pub bootstrap_bundle: crate::control::types::AgentBootstrapBundle,
    pub arm_linux_install_code: String,
    pub macos_install_code: String,
}

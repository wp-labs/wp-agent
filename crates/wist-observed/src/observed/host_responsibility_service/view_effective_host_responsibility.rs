// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Observed", module = "Observed.HostResponsibilityService")]
pub struct ViewEffectiveHostResponsibility {
    pub host_id: String,
    pub requested_by: String,
}

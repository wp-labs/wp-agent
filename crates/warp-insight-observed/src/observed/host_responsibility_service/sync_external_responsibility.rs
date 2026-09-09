// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "message", role = "command", domain = "Observed", module = "Observed.HostResponsibilityService")]
pub struct SyncExternalResponsibility {
    pub system_type: crate::ExternalSystemType,
    pub scope_key: String,
    pub requested_by: String,
}

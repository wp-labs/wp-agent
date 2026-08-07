// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostResponsibilityService")]
pub struct ExternalSyncCursor {
    #[moju(unique)]
    pub cursor_id: String,
    pub tenant_id: String,
    pub system_type: crate::ExternalSystemType,
    pub scope_key: String,
    pub cursor_value: Option<String>,
    pub full_sync_token: Option<String>,
    pub last_success_at: Option<crate::DateTime>,
    pub last_attempt_at: Option<crate::DateTime>,
    pub last_error: Option<String>,
    pub updated_at: crate::DateTime,
}

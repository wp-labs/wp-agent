// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostResponsibility")]
pub struct ExternalIdentityLink {
    #[moju(unique)]
    pub link_id: String,
    pub tenant_id: String,
    pub system_type: crate::ExternalSystemType,
    pub object_type: String,
    pub external_id: String,
    pub external_key: Option<String>,
    pub internal_kind: crate::ExternalLinkInternalKind,
    pub internal_id: String,
    pub status: String,
    pub first_seen_at: crate::DateTime,
    pub last_seen_at: crate::DateTime,
    pub last_synced_at: crate::DateTime,
}

// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostResponsibility")]
pub struct Subject {
    #[moju(unique)]
    pub subject_id: String,
    pub tenant_id: String,
    pub subject_type: crate::SubjectType,
    pub name: String,
    pub external_ref: Option<String>,
    pub status: String,
    pub created_at: crate::DateTime,
    pub updated_at: crate::DateTime,
}

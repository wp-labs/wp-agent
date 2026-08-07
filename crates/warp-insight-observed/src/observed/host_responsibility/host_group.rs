// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostResponsibility")]
pub struct HostGroup {
    #[moju(unique)]
    pub host_group_id: String,
    pub tenant_id: String,
    pub environment_id: String,
    pub name: String,
    pub group_type: crate::HostGroupType,
    pub parent_group_id: Option<String>,
    pub description: Option<String>,
    pub created_at: crate::DateTime,
    pub updated_at: crate::DateTime,
}

// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Observed", module = "Observed.HostResponsibility")]
pub struct HostGroupMembership {
    pub host_id: String,
    pub host_group_id: String,
    pub membership_role: String,
    pub source: crate::AssignmentSource,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
    pub created_at: crate::DateTime,
    pub updated_at: crate::DateTime,
}

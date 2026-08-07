// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostResponsibility")]
pub struct ResponsibilityAssignment {
    #[moju(unique)]
    pub assignment_id: String,
    pub tenant_id: String,
    pub target_type: crate::AssignmentTargetType,
    pub target_id: String,
    pub role: crate::AssignmentRole,
    pub subject_id: String,
    pub is_primary: bool,
    pub priority: i64,
    pub source: crate::AssignmentSource,
    pub valid_from: crate::DateTime,
    pub valid_to: Option<crate::DateTime>,
    pub remark: Option<String>,
    pub created_at: crate::DateTime,
    pub updated_at: crate::DateTime,
}

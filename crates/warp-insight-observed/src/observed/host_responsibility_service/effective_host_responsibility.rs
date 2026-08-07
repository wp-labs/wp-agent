// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Observed", module = "Observed.HostResponsibilityService")]
pub struct EffectiveHostResponsibility {
    #[moju(unique)]
    pub host_id: String,
    pub owner_subject_id: Option<String>,
    pub maintainer_subject_id: Option<String>,
    pub oncall_subject_id: Option<String>,
    pub security_owner_subject_id: Option<String>,
    pub owner_resolved_from: Option<String>,
    pub maintainer_resolved_from: Option<String>,
    pub oncall_resolved_from: Option<String>,
    pub security_owner_resolved_from: Option<String>,
    pub resolved_at: crate::DateTime,
}

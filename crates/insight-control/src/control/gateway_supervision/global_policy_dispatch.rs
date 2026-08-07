// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GatewaySupervision")]
pub struct GlobalPolicyDispatch {
    #[moju(unique)]
    pub dispatch_id: String,
    pub policy_version: String,
    pub target_count: i64,
    pub status: String,
    pub dispatched_at: crate::DateTime,
}

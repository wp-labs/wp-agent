// @jumo generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::jumo_derive::Jumo)]
#[jumo(kind = "struct", domain = "Control", module = "Control.Gateway.Supervision")]
pub struct GlobalPolicyDispatch {
    #[jumo(unique)]
    pub dispatch_id: String,
    pub policy_version: String,
    pub target_count: i64,
    pub status: String,
    pub dispatched_at: crate::DateTime,
}

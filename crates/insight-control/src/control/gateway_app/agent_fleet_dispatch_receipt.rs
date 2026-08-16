// @moju generated
#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize, ::moju_derive::MoJu)]
#[moju(kind = "struct", domain = "Control", module = "Control.GatewayApp.Application")]
pub struct AgentFleetDispatchReceipt {
    #[moju(unique)]
    pub dispatch_id: String,
    pub command_kind: String,
    pub target_count: i64,
    pub status: String,
    pub created_at: crate::DateTime,
}

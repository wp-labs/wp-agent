#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct GatewayListView {
    pub gateway_count: i64,
    pub online_count: i64,
    pub degraded_count: i64,
    pub offline_count: i64,
    pub updated_at: crate::DateTime,
}

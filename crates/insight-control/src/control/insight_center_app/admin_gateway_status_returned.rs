#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminGatewayStatusReturned {
    pub status: crate::GatewayRuntimeStatus,
}

#[derive(Debug, Clone, PartialEq, ::serde::Serialize, ::serde::Deserialize)]
pub struct AdminGatewayStatusListReturned {
    pub statuses: Vec<crate::GatewayRuntimeStatus>,
}
